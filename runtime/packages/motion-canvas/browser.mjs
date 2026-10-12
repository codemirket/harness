// Fixed authoring adapter: upstream Motion Canvas owns geometry, signals and frame playback.
import {PlaybackManager, PlaybackState, PlaybackStatus, Stage, SharedWebGLContext,
  Logger, Vector2, Color, createSignal, all, delay, waitFor, easeInOutCubic,
  easeOutCubic, easeOutBack, linear} from '@motion-canvas/core';
import {ReadOnlyTimeEvents} from '@motion-canvas/core/lib/scenes/timeEvents';
import {makeScene2D, Rect, Circle, Path, Txt, Node, Gradient} from '@motion-canvas/2d';

export async function initialise(data) {
  const font = new FontFace('Mirket Inter', 'url(/font.woff2)', {weight: '100 900'});
  await font.load();
  document.fonts.add(font);
  await document.fonts.ready;
  const logger = new Logger();
  const errors = [];
  logger.onLogged.subscribe(message => {
    if (message.level === 'error' || message.level === 'warn') {
      if (errors.length < 16) errors.push(String(message.message || message).slice(0, 600));
    }
  });
  let nodes = new Map();
  let gradients = new Map();
  const classes = {rect: Rect, ellipse: Circle, path: Path, text: Txt, group: Node};
  const easings = {linear, easeInOutCubic, easeOutCubic, easeOutBack};
  const description = makeScene2D(function* (view) {
    nodes = new Map();
    gradients = new Map();
    for (const definition of data.gradients || []) {
      const signals = definition.stops.map(stop => ({offset: createSignal(stop.offset),
        color: createSignal(new Color(stop.color), Color.lerp)}));
      const gradient = new Gradient({...definition, id: undefined, stops: signals});
      gradients.set(definition.id, {gradient, signals});
    }
    for (const definition of data.nodes) {
      const props = {...definition.props, key: definition.id};
      for (const field of ['fill', 'stroke']) {
        if (props[field] && typeof props[field] === 'object') {
          props[field] = gradients.get(props[field].gradient).gradient;
        }
      }
      if (definition.type === 'text') props.fontFamily = 'Mirket Inter';
      const node = new classes[definition.type](props);
      (definition.parent ? nodes.get(definition.parent) : view).add(node);
      nodes.set(definition.id, node);
    }
    function* animate(track) {
      const target = track.target.startsWith('gradient:')
        ? gradients.get(track.target.slice(9)) : nodes.get(track.target);
      const stop = track.property.match(/^stop\.(\d+)\.(color|offset)$/);
      const signal = stop ? target.signals[+stop[1]][stop[2]]
        : track.target.startsWith('gradient:') ? target.gradient[track.property] : target[track.property];
      const value = stop && stop[2] === 'color' ? new Color(track.to) : track.to;
      yield* signal(value, (track.end - track.start) / data.fps, easings[track.easing || 'easeInOutCubic']);
    }
    yield* all(waitFor((data.frames + 1) / data.fps),
      ...(data.tracks || []).map(track => delay(track.start / data.fps, animate(track))));
  });
  description.meta.seed.set(0);
  const playback = new PlaybackManager();
  playback.fps = data.fps;
  playback.state = PlaybackState.Rendering;
  const sharedWebGLContext = new SharedWebGLContext(logger);
  const scene = new description.klass({...description, name: 'mirket-vector',
    size: new Vector2(data.width, data.height), resolutionScale: 1,
    logger, playback: new PlaybackStatus(playback), timeEventsClass: ReadOnlyTimeEvents,
    sharedWebGLContext, experimentalFeatures: false});
  playback.setup([scene]);
  const stage = new Stage();
  stage.configure({size: new Vector2(data.width, data.height), resolutionScale: 1,
    background: data.background ?? null, colorSpace: 'srgb'});
  document.body.append(stage.finalBuffer);
  await playback.recalculate();
  if (playback.duration < data.frames) throw new Error('Motion Canvas scene ended before the requested duration.');
  await playback.reset();
  function clean() { if (errors.length) throw new Error(errors.join('\n')); }
  window.__mirketMotionCanvas = {
    async capture(frame) {
      await playback.seek(frame);
      if (playback.frame !== frame) throw new Error(`Motion Canvas reached ${playback.frame}, expected ${frame}.`);
      await stage.render(playback.currentScene, playback.previousScene);
      clean();
      return stage.finalBuffer.toDataURL('image/png');
    },
    inspect() {
      clean();
      return scene.execute(() => ({runtime: 'Motion Canvas', version: '3.17.2',
        playback_api: 'Scene2D+PlaybackManager+Stage', duration_frames: playback.duration,
        actual_frame: playback.frame, node_count: nodes.size, nodes_truncated: nodes.size > 64, nodes: [...nodes].slice(0, 64).map(([id, node]) => ({id,
          type: node.constructor.name, position: node.position().serialize(),
          rotation: node.rotation(), scale: node.scale().serialize(), opacity: node.opacity(),
          ...(node.text ? {text: node.text().slice(0, 256), text_length: node.text().length, font: node.fontFamily()} : {}),
          ...(node.data ? {path: node.data().slice(0, 256), path_length: node.data().length} : {}),
          ...(node.clip ? {clip: node.clip()} : {})})),
        gradients: [...gradients].map(([id, value]) => ({id, type: value.gradient.type(),
          from: value.gradient.from().serialize(), to: value.gradient.to().serialize(),
          stops: value.signals.map(stop => ({offset: stop.offset(), color: stop.color().serialize()}))})),
        track_count: (data.tracks || []).length, font: {family: font.family, status: font.status}}));
    },
  };
  clean();
}
