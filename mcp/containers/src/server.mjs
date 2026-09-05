import { McpServer } from '@modelcontextprotocol/server';
import { z } from 'zod';
import { nameSchema, templateSchema } from './catalog.mjs';

export function createServer(manager) {
  const server = new McpServer({ name: 'containers', version: '1.0.0' }, {
    instructions: 'Manage shared local development services. First list_services and list_catalog; reuse an existing compatible service. ensure_service starts or creates it. provision_project allocates separate PostgreSQL databases, Redis ACL prefixes, or MinIO buckets. get_connection explicitly returns secrets: use them only for local development, never commit or log them. stop_service and remove_service affect ALL projects; obtain user authorization and pass confirmAlias. remove_service retains all data. Custom image/command inputs are executable software and must follow user intent. The server never deletes bind data or manages unowned containers.',
  });
  const tool = (name, description, inputSchema, handler, readOnly = false, destructive = false) => {
    server.registerTool(name, { description, inputSchema, annotations: { readOnlyHint: readOnly, destructiveHint: destructive, idempotentHint: name !== 'add_catalog_entry', openWorldHint: !readOnly } }, async args => {
      try {
        const value = await handler(args);
        return { content: [{ type: 'text', text: JSON.stringify(value, null, 2) }], structuredContent: value };
      } catch (error) {
        return { isError: true, content: [{ type: 'text', text: error.message }] };
      }
    });
  };
  tool('list_catalog', 'List available local service templates without credentials. The initial catalog includes PostgreSQL, Redis and MinIO.', z.object({}), () => ({ catalog: manager.listCatalog() }), true);
  tool('list_services', 'Discover all local Docker containers and managed reusable services, including stopped containers and project allocations. Does not reveal credentials.', z.object({}), () => manager.listServices(), true);
  tool('inspect_service', 'Inspect a managed service, actual host ports, Docker network, persistence location, and projects.', z.object({ alias: nameSchema }), a => manager.inspectService(a.alias), true);
  tool('add_catalog_entry', 'Add a reusable Docker service template. Bind storage is always under ~/.dev.volumes/<alias>. secretEnvironment names receive generated passwords. No host paths, privileged flags, or arbitrary Docker arguments are accepted. Ordinary environment values are stored privately; use secretEnvironment for generated secrets.', templateSchema, a => manager.addTemplate(a));
  tool('remove_catalog_entry', 'Remove an unused catalog template. Active service references must be removed first. Container data is unaffected.', z.object({ id: nameSchema }), a => manager.removeTemplate(a.id), false, true);
  tool('ensure_service', 'Reuse/start an existing service or create one from a catalog template. Use the same alias across projects. Ports bind only to 127.0.0.1. hostPorts maps port names to host ports; 0 requests an available port. Image pulls/MinIO source build can take several minutes. Data and credentials persist.', z.object({ template: nameSchema, alias: nameSchema.optional(), hostPorts: z.record(nameSchema, z.number().int().min(0).max(65535)).default({}) }), a => manager.ensureService(a.template, a.alias ?? a.template, a.hostPorts));
  tool('provision_project', 'Allocate a project on a running shared PostgreSQL, Redis, or MinIO service. Repeated calls reuse its database/user, key prefix/user, or bucket/user. Project IDs are stable across agents; do not use filesystem paths.', z.object({ alias: nameSchema, project: nameSchema }), a => manager.project(a.alias, a.project));
  tool('get_connection', 'Explicitly retrieve sensitive local development credentials and actual endpoints. Supply project for project-scoped credentials; omitting it returns administrator access shared by all projects. Never log or commit this result.', z.object({ alias: nameSchema, project: nameSchema.optional() }), a => manager.connection(a.alias, a.project), true);
  tool('stop_service', 'Stop a shared container, interrupting ALL its projects. Obtain user authorization and repeat alias in confirmAlias. Data is retained.', z.object({ alias: nameSchema, confirmAlias: nameSchema }), a => manager.stopService(a.alias, a.confirmAlias), false, true);
  tool('remove_service', 'Remove a shared container after user authorization; repeat alias in confirmAlias. Retains its bind directory, credentials and project allocations. ensure_service recreates it using the retained data.', z.object({ alias: nameSchema, confirmAlias: nameSchema }), a => manager.removeService(a.alias, a.confirmAlias), false, true);
  server.registerResource('catalog', 'containers://catalog', { description: 'Available reusable local service templates', mimeType: 'application/json' }, async uri => ({ contents: [{ uri: uri.href, mimeType: 'application/json', text: JSON.stringify(manager.listCatalog()) }] }));
  server.registerPrompt('use-local-service', { description: 'Find or provision a shared local development dependency', argsSchema: { service: z.string(), project: z.string() } }, ({ service, project }) => ({ messages: [{ role: 'user', content: { type: 'text', text: `Use containers MCP to find a reusable ${service} service for project ${project}. List the existing services and catalog first, reuse a compatible alias, ensure it is running, provision the project, and retrieve its project connection. Keep credentials out of source control. Do not stop or remove shared services without my explicit request.` } }] }));
  return server;
}
