"""Build a bitmap-only profile from the exact pinned ComfyUI source tree."""

import ast
from pathlib import Path
import re
import sys


EXTRAS = (
    "nodes_stable3d.py",
    "nodes_load_3d.py",
    "nodes_hunyuan3d.py",
    "nodes_camera_trajectory.py",
    "nodes_trellis2.py",
    "nodes_mesh_postprocess.py",
    "nodes_save_3d.py",
    "nodes_mesh_io.py",
    "nodes_moge.py",
    "nodes_gaussian_splat.py",
    "nodes_triposplat.py",
    "nodes_depth_anything_3.py",
    "nodes_sam3d_body.py",
    "nodes_glsl.py",
)

PARTNER_NODES = (
    "nodes_hunyuan3d.py",
    "nodes_meshy.py",
    "nodes_rodin.py",
    "nodes_tripo.py",
)

PARTNER_CLIENTS = ("hunyuan3d.py", "meshy.py", "tripo.py", "rodin.py")
SUPPORT_DIRS = ("mesh3d", "sam3d_body")
CORE_DIRS = ("hunyuan3d", "hunyuan3dv2_1", "sam3d_body", "trellis2", "triposplat", "moge")
THREE_D_BLUEPRINTS = (
    "Image to Model (Hunyuan3d 2.1).json",
    "Image to Gaussian Splat (TripoSplat).json",
    "Geometry Estimation (MoGe).json",
    "Image Depth Estimation (MoGe).json",
    "Video Depth Estimation (MoGe).json",
    "Image Depth Estimation (Depth Anything 3).json",
    "Video Depth Estimation (Depth Anything 3).json",
    "Video Edit (Bernini-R).json",
)
THREE_D_IO = (
    "Voxel", "Mesh", "Splat", "File3DAny", "File3DGLB", "File3DGLTF", "File3DFBX",
    "File3DOBJ", "File3DSTL", "File3DUSDZ", "File3DPLY", "File3DSPLAT", "File3DSPZ",
    "File3DKSPLAT", "File3DSplatAny", "File3DPointCloudAny", "Load3DCamera",
    "Load3DModelInfo", "Load3D", "Load3DAnimation",
)
RODIN_MODELS = (
    "Rodin3DCheckStatusRequest", "Rodin3DDownloadRequest", "RodinGenerateJobsData",
    "RodinMaterialType", "RodinMeshModeType", "RodinQualityType", "RodinResourceItem",
    "RodinStatusOptions", "RodinTierType", "Rodin3DDownloadResponse",
    "Rodin3DGenerateRequest", "Rodin3DGenerateResponse", "RodinCheckStatusJobItem",
    "Rodin3DCheckStatusResponse",
)


def regular_file(path: Path) -> None:
    if not path.is_file() or path.is_symlink():
        raise RuntimeError(f"expected regular pinned-source file: {path}")


def remove_unique_line(source: str, line: str) -> str:
    if source.count(line) != 1:
        raise RuntimeError(f"expected one pinned-source line: {line.strip()}")
    return source.replace(line, "")


def replace_unique(source: str, old: str, new: str) -> str:
    if source.count(old) != 1:
        raise RuntimeError(f"expected one pinned-source block: {old[:80].strip()}")
    return source.replace(old, new)


def remove_span(source: str, start: str, end: str) -> str:
    if source.count(start) != 1 or source.count(end) != 1:
        raise RuntimeError(f"unexpected pinned-source span: {start.strip()}")
    left = source.index(start)
    right = source.index(end)
    if left >= right:
        raise RuntimeError(f"reordered pinned-source span: {start.strip()}")
    return source[:left] + source[right:]


def remove_top_level_classes(source: str, names: tuple[str, ...]) -> str:
    tree = ast.parse(source)
    classes = [node for node in tree.body if isinstance(node, ast.ClassDef) and node.name in names]
    if len(classes) != len(names) or {node.name for node in classes} != set(names):
        raise RuntimeError(f"unexpected pinned-source classes: {names}")
    lines = source.splitlines(keepends=True)
    for node in sorted(classes, key=lambda entry: entry.lineno, reverse=True):
        start = min((decorator.lineno for decorator in node.decorator_list), default=node.lineno)
        del lines[start - 1 : node.end_lineno]
    result = "".join(lines)
    ast.parse(result)
    return result


def remove_class_methods(source: str, class_name: str, names: tuple[str, ...]) -> str:
    tree = ast.parse(source)
    classes = [node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == class_name]
    if len(classes) != 1:
        raise RuntimeError(f"expected pinned-source class: {class_name}")
    methods = [node for node in classes[0].body if isinstance(node, ast.FunctionDef) and node.name in names]
    if len(methods) != len(names) or {node.name for node in methods} != set(names):
        raise RuntimeError(f"unexpected pinned-source methods: {class_name}.{names}")
    lines = source.splitlines(keepends=True)
    for node in sorted(methods, key=lambda entry: entry.lineno, reverse=True):
        start = min((decorator.lineno for decorator in node.decorator_list), default=node.lineno)
        del lines[start - 1 : node.end_lineno]
    result = "".join(lines)
    ast.parse(result)
    return result


def remove_directory(path: Path) -> None:
    if not path.is_dir() or path.is_symlink():
        raise RuntimeError(f"expected pinned-source directory: {path}")
    for entry in sorted(path.rglob("*"), reverse=True):
        if entry.is_symlink():
            raise RuntimeError(f"unexpected symlink in pinned source: {entry}")
        if entry.is_file():
            entry.unlink()
        elif entry.is_dir():
            entry.rmdir()
    path.rmdir()


def prune_image_encoders(root: Path) -> None:
    clip = root / "comfy/clip_vision.py"
    dino = root / "comfy/image_encoders/dino2.py"
    naf = root / "comfy/image_encoders/naf.py"
    for path in (clip, dino, naf):
        regular_file(path)

    source = clip.read_text()
    source = remove_unique_line(source, "from comfy.image_encoders.naf import NAF\n")
    source = remove_unique_line(source, "        self.naf = None\n")
    source = remove_span(source, "    # NAF feature upsampler bundled into the DINOv3 file under the `naf.` prefix.\n",
                         "    return clip\n")
    ast.parse(source)
    clip.write_text(source)
    naf.unlink()

    source = dino.read_text()
    source = remove_unique_line(source, "import torch.nn.functional as F\n")
    source = remove_span(source, "from comfy.ldm.depth_anything_3.reference_view_selector import (\n",
                         "\n\nclass Dino2AttentionOutput(torch.nn.Module):")
    source = remove_top_level_classes(source, ("_PositionGetter", "RotaryPositionEmbedding2D"))
    source = remove_class_methods(source, "Dinov2Model", (
        "_prepare_rope_positions", "_inject_camera_token", "get_intermediate_layers_da3"
    ))
    source = replace_unique(source,
        "    def __init__(self, embed_dim, heads, layer_norm_eps, dtype, device, operations,\n"
        "                 qk_norm=False):\n",
        "    def __init__(self, embed_dim, heads, layer_norm_eps, dtype, device, operations):\n")
    source = remove_span(source, "        if qk_norm:\n", "\n    def forward(self, x, mask, optimized_attention, pos=None, rope=None):")
    source = replace_unique(source,
        "    def forward(self, x, mask, optimized_attention, pos=None, rope=None):\n"
        "        # Fast path used by the existing CLIP-vision DINOv2 (no DA3 extensions).\n"
        "        if self.q_norm is None and rope is None:\n"
        "            return self.output(self.attention(x, mask, optimized_attention))\n\n"
        "        # DA3 path: do QKV manually so we can apply per-head QK-norm and 2D RoPE.\n"
        "        attn = self.attention\n"
        "        B, N, C = x.shape\n"
        "        h = self.heads\n"
        "        d = self.head_dim\n"
        "        q = attn.query(x).view(B, N, h, d).transpose(1, 2)\n"
        "        k = attn.key(x).view(B, N, h, d).transpose(1, 2)\n"
        "        v = attn.value(x).view(B, N, h, d).transpose(1, 2)\n"
        "        if self.q_norm is not None:\n"
        "            q = self.q_norm(q)\n"
        "            k = self.k_norm(k)\n"
        "        if rope is not None and pos is not None:\n"
        "            q = rope(q, pos)\n"
        "            k = rope(k, pos)\n"
        "        out = optimized_attention(q, k, v, h, mask=mask, skip_reshape=True)\n"
        "        return self.output(out)\n",
        "    def forward(self, x, mask, optimized_attention):\n"
        "        return self.output(self.attention(x, mask, optimized_attention))\n")
    source = replace_unique(source,
        "    def __init__(self, dim, num_heads, layer_norm_eps, dtype, device, operations, use_swiglu_ffn,\n"
        "                 qk_norm=False):\n",
        "    def __init__(self, dim, num_heads, layer_norm_eps, dtype, device, operations, use_swiglu_ffn):\n")
    source = replace_unique(source,
        "        self.attention = Dino2AttentionBlock(dim, num_heads, layer_norm_eps, dtype, device, operations,\n"
        "                                             qk_norm=qk_norm)\n",
        "        self.attention = Dino2AttentionBlock(dim, num_heads, layer_norm_eps, dtype, device, operations)\n")
    source = replace_unique(source,
        "    def forward(self, x, optimized_attention, pos=None, rope=None, attn_mask=None):\n"
        "        x = x + self.layer_scale1(self.attention(self.norm1(x), attn_mask, optimized_attention,\n"
        "                                                 pos=pos, rope=rope))\n",
        "    def forward(self, x, optimized_attention, attn_mask=None):\n"
        "        x = x + self.layer_scale1(self.attention(self.norm1(x), attn_mask, optimized_attention))\n")
    source = remove_span(source,
                         "# -----------------------------------------------------------------------------\n# 2D Rotary position embedding (DA3 extension)\n",
                         "class Dino2Encoder(torch.nn.Module):")
    source = replace_unique(source,
        "    def __init__(self, dim, num_heads, layer_norm_eps, num_layers, dtype, device, operations, use_swiglu_ffn,\n"
        "                 qknorm_start: int = -1):\n",
        "    def __init__(self, dim, num_heads, layer_norm_eps, num_layers, dtype, device, operations, use_swiglu_ffn):\n")
    source = replace_unique(source,
        "                use_swiglu_ffn=use_swiglu_ffn,\n"
        "                qk_norm=(qknorm_start != -1 and i >= qknorm_start),\n",
        "                use_swiglu_ffn=use_swiglu_ffn,\n")
    source = remove_unique_line(source,
        "        # Backward-compat path used by ``ClipVisionModel`` (no DA3 extensions).\n")
    source = replace_unique(source,
        "                 use_mask_token: bool = True,\n"
        "                 num_camera_tokens: int = 0):\n",
        "                 use_mask_token: bool = True):\n")
    source = remove_span(source, "        if num_camera_tokens > 0:\n",
                         "\n    def interpolate_pos_encoding(self, x, h_pixels, w_pixels):")
    source = remove_unique_line(source,
        "        # non-square paths like DA3-Small / DA3-Base multi-view.\n")
    source = remove_span(source, "        # DA3 extensions (all default to disabled).\n",
                         "        self.embed_dim = dim\n")
    source = remove_span(source, "        if self.rope_start != -1 and rope_freq > 0:\n",
                         "        self.embeddings = Dino2Embeddings(\n")
    source = replace_unique(source,
        "            use_mask_token=use_mask_token, num_camera_tokens=num_cam_tokens,\n",
        "            use_mask_token=use_mask_token,\n")
    source = replace_unique(source,
        "            use_swiglu_ffn=use_swiglu_ffn,\n"
        "            qknorm_start=self.qknorm_start,\n",
        "            use_swiglu_ffn=use_swiglu_ffn,\n")
    source = replace_unique(source,
        "        if self.alt_start != -1:\n"
        "            raise RuntimeError(\n"
        "                \"Dinov2Model.forward() is the backward-compatible CLIP-vision path and does not \"\n"
        "                \"apply DA3 extensions (RoPE, alternating attention, camera-token injection). \"\n"
        "                \"Use get_intermediate_layers_da3() for Depth Anything 3 models.\"\n"
        "            )\n",
        "")
    # Method removal leaves only a section heading and whitespace at the end.
    source = replace_unique(source,
        "    # ------------------------------------------------------------------\n"
        "    # Depth Anything 3 forward\n"
        "    # ------------------------------------------------------------------\n",
        "")
    ast.parse(source)
    dino.write_text(source)


def prune_api(root: Path) -> None:
    io = root / "comfy_api/latest/_io.py"
    util_init = root / "comfy_api/latest/_util/__init__.py"
    latest_init = root / "comfy_api/latest/__init__.py"
    geometry = root / "comfy_api/latest/_util/geometry_types.py"
    generated = root / "comfy_api_nodes/apis/__init__.py"
    replacements = root / "comfy_extras/nodes_replacements.py"
    images = root / "comfy_extras/nodes_images.py"
    util_exports = root / "comfy_api_nodes/util/__init__.py"
    uploads = root / "comfy_api_nodes/util/upload_helpers.py"
    downloads = root / "comfy_api_nodes/util/download_helpers.py"
    folders = root / "folder_paths.py"
    for path in (io, util_init, latest_init, geometry, generated, replacements, images,
                 util_exports, uploads, downloads, folders):
        regular_file(path)

    source = io.read_text()
    source = remove_top_level_classes(source, THREE_D_IO)
    source = replace_unique(source,
        "from ._util import MESH, VOXEL, SPLAT, SVG as _SVG, File3D\n",
        "from ._util import SVG as _SVG\n")
    source = replace_unique(source,
        "        '''Only needed if dealing with these types: audio, hunyuan3dv2'''\n",
        "        '''Only needed if dealing with audio.'''\n")
    for line in (
        "        elevation: NotRequired[list[float]]\n",
        "        azimuth: NotRequired[list[float]]\n",
        "        '''Used by SV3D.'''\n",
    ):
        if line == "        '''Used by SV3D.'''\n":
            if source.count(line) != 2:
                raise RuntimeError("expected two pinned SV3D field comments")
            source = source.replace(line, "")
        else:
            source = remove_unique_line(source, line)
    for name in THREE_D_IO:
        source = remove_unique_line(source, f'    "{name}",\n')
    ast.parse(source)
    io.write_text(source)

    source = util_init.read_text()
    source = remove_unique_line(source,
        "from .geometry_types import VOXEL, MESH, SPLAT, File3D\n")
    for name in ("VOXEL", "MESH", "SPLAT", "File3D"):
        source = remove_unique_line(source, f'    "{name}",\n')
    util_init.write_text(source)
    geometry.unlink()

    source = latest_init.read_text()
    source = replace_unique(source,
        "from ._util import VideoCodec, VideoContainer, VideoComponents, MESH, VOXEL, SPLAT, File3D\n",
        "from ._util import VideoCodec, VideoContainer, VideoComponents\n")
    for name in ("MESH", "VOXEL", "SPLAT", "File3D"):
        source = remove_unique_line(source, f"    {name} = {name}\n")
    ast.parse(source)
    latest_init.write_text(source)

    source = generated.read_text()
    source = remove_top_level_classes(source, RODIN_MODELS)
    generated.write_text(source)

    source = replacements.read_text()
    source = remove_unique_line(source, "    await register_replacements_load3d()\n")
    source = remove_unique_line(source, "    await register_replacements_preview3d()\n")
    source = remove_span(source, "async def register_replacements_load3d():\n",
                         "async def register_replacements_svdimg2vid():\n")
    ast.parse(source)
    replacements.write_text(source)

    source = images.read_text()
    source = replace_unique(source,
        'logging.info("Trellis2 preprocess: mask bbox empty, using inverted mask.")',
        'logging.info("Image mask bbox empty, using inverted mask.")')
    source = replace_unique(source,
        "    with zero-padding. Useful for 3D pipelines that expect a centered, background-free\n"
        "    subject at a fixed input resolution (Trellis2, Pixal3D, Hunyuan3D, TripoSR, etc.).\"\"\"",
        "    with zero-padding. Produces a centered subject on a fixed-size bitmap canvas.\"\"\"")
    source = replace_unique(source,
        '"crop mask resize", "trellis2", "pixal3d"',
        '"crop mask resize"')
    ast.parse(source)
    images.write_text(source)

    source = uploads.read_text()
    source = remove_span(source, "_3D_MIME_TYPES = {\n",
                         "async def upload_file_to_comfyapi(\n")
    ast.parse(source)
    uploads.write_text(source)

    source = downloads.read_text()
    source = remove_unique_line(source, "from folder_paths import get_output_directory\n")
    source = replace_unique(source,
        "from comfy_api.latest import InputImpl, Types\n",
        "from comfy_api.latest import InputImpl\n")
    tail = "    return Types.File3D(source=data, file_format=file_format)\n"
    if not source.endswith(tail) or source.count("async def download_url_to_file_3d(\n") != 1:
        raise RuntimeError("unexpected pinned-source 3D download helper")
    source = source[:source.index("async def download_url_to_file_3d(\n")]
    ast.parse(source)
    downloads.write_text(source)

    source = util_exports.read_text()
    for line in (
        "    download_url_to_file_3d,\n",
        "    upload_3d_model_to_comfyapi,\n",
        '    "download_url_to_file_3d",\n',
        '    "upload_3d_model_to_comfyapi",\n',
    ):
        source = remove_unique_line(source, line)
    ast.parse(source)
    util_exports.write_text(source)

    source = folders.read_text()
    source = remove_unique_line(source,
        'folder_names_and_paths["geometry_estimation"] = ([os.path.join(models_dir, "geometry_estimation")], supported_pt_extensions)\n')
    source = remove_unique_line(source, '    "fbx" : "model",\n')
    source = replace_unique(source,
        'content_types: List[Literal["image", "video", "audio", "model"]]',
        'content_types: List[Literal["image", "video", "audio"]]')
    source = replace_unique(source,
        "    Note:\n"
        "        - 'model' in MIME context refers to 3D models, not files containing trained weights and parameters\n",
        "")
    ast.parse(source)
    folders.write_text(source)


def prune_headless(root: Path) -> None:
    server = root / "server.py"
    frontend = root / "app/frontend_management.py"
    requirements = root / "requirements.txt"
    for path in (server, frontend, requirements):
        regular_file(path)

    source = requirements.read_text()
    for line in source.splitlines(keepends=True):
        if line.startswith((
            "comfyui-frontend-package==",
            "comfyui-workflow-templates==",
            "comfyui-embedded-docs==",
            "PyOpenGL>=",
            "comfy-angle",
        )):
            source = remove_unique_line(source, line)
    if any(name in source for name in (
        "comfyui-frontend-package", "comfyui-workflow-templates", "comfyui-embedded-docs",
        "PyOpenGL", "comfy-angle"
    )):
        raise RuntimeError("unreviewed headless dependency remained in pinned requirements")
    requirements.write_text(source)

    source = server.read_text()
    source = remove_unique_line(source,
        "from app.frontend_management import FrontendManager, parse_version\n")
    source = replace_unique(source,
        "        self.web_root = (\n"
        "            FrontendManager.init_frontend(args.front_end_version)\n"
        "            if args.front_end_root is None\n"
        "            else args.front_end_root\n"
        "        )\n",
        "        if args.front_end_root is None:\n"
        "            raise RuntimeError('headless ComfyUI requires --front-end-root')\n"
        "        self.web_root = args.front_end_root\n")
    for line in (
        "            required_frontend_version = FrontendManager.get_required_frontend_version()\n",
        "            installed_templates_version = FrontendManager.get_installed_templates_version()\n",
        "            required_templates_version = FrontendManager.get_required_templates_version()\n",
        "            comfy_package_versions = FrontendManager.get_comfy_package_versions()\n",
        '                    "required_frontend_version": required_frontend_version,\n',
        '                    "installed_templates_version": installed_templates_version,\n',
        '                    "required_templates_version": required_templates_version,\n',
        '                    "comfy_package_versions": comfy_package_versions,\n',
    ):
        source = remove_unique_line(source, line)
    source = remove_span(source,
        "        installed_templates_version = FrontendManager.get_installed_templates_version()\n",
        "        self.app.add_routes([\n            web.static('/', self.web_root),\n        ])\n")
    ast.parse(source)
    server.write_text(source)
    frontend.unlink()


def prune_core(root: Path) -> None:
    sd = root / "comfy/sd.py"
    model_base = root / "comfy/model_base.py"
    detection = root / "comfy/model_detection.py"
    supported = root / "comfy/supported_models.py"
    latent = root / "comfy/latent_formats.py"
    for path in (sd, model_base, detection, supported, latent):
        regular_file(path)

    source = sd.read_text()
    for line in (
        "import comfy.ldm.trellis2.vae\n",
        "import comfy.ldm.hunyuan3d.vae\n",
        "import comfy.ldm.triposplat.vae\n",
    ):
        source = remove_unique_line(source, line)
    source = remove_span(source, '            elif "shape_dec.blocks.1.16.to_subdiv.weight" in sd:',
                         '            elif "decoder.up_blocks.2.upsamplers.0.upscale_conv.weight" in sd:')
    source = remove_span(source, "            # Hunyuan 3d v2 2.0 & 2.1\n",
                         '            elif "vocoder.backbone.channel_layers.0.0.bias" in sd:')
    source = remove_span(source, '            elif "gs.base_offset_scale" in sd and "octree.out_proj.weight" in sd:',
                         '            else:\n                logging.warning("WARNING: No VAE weights detected')
    ast.parse(source)
    sd.write_text(source)

    source = model_base.read_text()
    for line in (
        "import comfy.ldm.hunyuan3dv2_1\n",
        "import comfy.ldm.hunyuan3dv2_1.hunyuandit\n",
        "import comfy.ldm.hunyuan3d.model\n",
        "import comfy.ldm.triposplat.model\n",
        "import comfy.ldm.trellis2.model\n",
        "import comfy.ldm.depth_anything_3.model\n",
    ):
        source = remove_unique_line(source, line)
    source = remove_top_level_classes(source, (
        "Trellis2", "Hunyuan3Dv2", "Hunyuan3Dv2_1", "TripoSplat", "DepthAnything3",
        "SV3D_u", "SV3D_p", "Stable_Zero123"
    ))
    model_base.write_text(source)

    source = detection.read_text()
    source = remove_span(source, "    shape_key = '{}img2shape.t_embedder.mlp.0.weight'.format(key_prefix)",
                         "    if '{}transformer.rotary_pos_emb.inv_freq'.format(key_prefix) in state_dict_keys:")
    source = remove_span(source, "    if '{}latent_in.weight'.format(key_prefix) in state_dict_keys:  # Hunyuan 3D",
                         "    if '{}t_embedder1.mlp.0.weight'.format(key_prefix) in state_dict_keys and '{}x_embedder.proj1.weight'.format(key_prefix) in state_dict_keys:")
    source = remove_span(source, "    # Depth Anything 3 (repackaged to ComfyUI's native Dinov2Model layout via scripts/convert_da3.py)",
                         "    if '{}layers.0.mlp.linear_fc2.weight'.format(key_prefix) in state_dict_keys:")
    ast.parse(source)
    detection.write_text(source)

    source = supported.read_text()
    source = remove_top_level_classes(source, (
        "Trellis2", "Hunyuan3Dv2", "Hunyuan3Dv2_1", "Hunyuan3Dv2mini", "TripoSplat", "DepthAnything3",
        "SV3D_u", "SV3D_p", "Stable_Zero123"
    ))
    for name in ("Trellis2", "Hunyuan3Dv2", "Hunyuan3Dv2_1", "Hunyuan3Dv2mini", "TripoSplat", "DepthAnything3",
                 "SV3D_u", "SV3D_p", "Stable_Zero123"):
        source = remove_unique_line(source, f"    {name},\n")
    ast.parse(source)
    supported.write_text(source)

    source = latent.read_text()
    source = remove_top_level_classes(source, (
        "TripoSplat", "Trellis2", "Trellis2SLAT", "Trellis2ShapeSLAT", "Trellis2TexSLAT",
        "Hunyuan3Dv2", "Hunyuan3Dv2_1", "Hunyuan3Dv2mini"
    ))
    latent.write_text(source)

    for name in CORE_DIRS:
        remove_directory(root / "comfy/ldm" / name)
    prune_image_encoders(root)
    remove_directory(root / "comfy/ldm/depth_anything_3")
    for name in THREE_D_BLUEPRINTS:
        path = root / "blueprints" / name
        regular_file(path)
        path.unlink()


def main(root: Path) -> None:
    root = root.resolve(strict=True)
    node_registry = root / "nodes.py"
    regular_file(node_registry)
    source = node_registry.read_text()
    if "async def init_builtin_extra_nodes():" not in source or "async def init_builtin_api_nodes():" not in source:
        raise RuntimeError("unexpected pinned ComfyUI node registration structure")

    for name in EXTRAS:
        registration = f'        "{name}",\n'
        if source.count(registration) != 1:
            raise RuntimeError(f"expected exactly one built-in registration: {name}")
        regular_file(root / "comfy_extras" / name)
        source = source.replace(registration, "")

    for name in PARTNER_NODES:
        regular_file(root / "comfy_api_nodes" / name)
    for name in PARTNER_CLIENTS:
        regular_file(root / "comfy_api_nodes" / "apis" / name)
    for name in SUPPORT_DIRS:
        directory = root / "comfy_extras" / name
        if not directory.is_dir() or directory.is_symlink():
            raise RuntimeError(f"expected pinned-source support directory: {directory}")

    # The registration list and all target files were checked before any mutation.
    node_registry.write_text(source)
    for name in EXTRAS:
        (root / "comfy_extras" / name).unlink()
    for name in PARTNER_NODES:
        (root / "comfy_api_nodes" / name).unlink()
    for name in PARTNER_CLIENTS:
        (root / "comfy_api_nodes" / "apis" / name).unlink()
    for name in SUPPORT_DIRS:
        remove_directory(root / "comfy_extras" / name)

    prune_core(root)
    prune_api(root)
    prune_headless(root)
    remove_directory(root / "models/geometry_estimation")
    # Repository tests and examples are not served in the runtime image.
    for name in ("tests", "tests-unit", "script_examples"):
        remove_directory(root / name)

    remaining = [node_registry]
    remaining.extend((root / "comfy_extras").rglob("*.py"))
    remaining.extend((root / "comfy_api_nodes").rglob("*.py"))
    remaining.extend((root / "comfy").rglob("*.py"))
    remaining.extend((root / "comfy_api").rglob("*.py"))
    remaining.extend((root / "app").rglob("*.py"))
    remaining.append(root / "server.py")
    disallowed = re.compile(
        r"comfy_extras\.(?:nodes_(?:save_3d|mesh_postprocess)|mesh3d|sam3d_body)"
        r"|comfy_api_nodes\.apis\.(?:hunyuan3d|meshy|tripo)"
        r"|comfy\.ldm\.(?:hunyuan3d|hunyuan3dv2_1|sam3d_body|trellis2|triposplat|moge)"
        r"|comfy\.ldm\.depth_anything_3"
    )
    for path in remaining:
        if disallowed.search(path.read_text()):
            raise RuntimeError(f"retained module imports excluded 3D source: {path}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: exclude-3d.py /opt/comfyui")
    main(Path(sys.argv[1]))
