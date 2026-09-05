import { z } from 'zod';

export const nameSchema = z.string().regex(/^[a-z][a-z0-9-]{0,39}$/, 'Use 1–40 lowercase letters, digits and hyphens, starting with a letter');
export const projectSchema = nameSchema;
const portSchema = z.object({
  name: nameSchema,
  container: z.number().int().min(1).max(65535),
  host: z.number().int().min(0).max(65535).default(0),
}).strict();
export const templateSchema = z.object({
  id: nameSchema,
  description: z.string().min(1).max(500),
  image: z.string().regex(/^[a-zA-Z0-9][a-zA-Z0-9._/:@-]+$/).max(240),
  dataPath: z.string().regex(/^\/[a-zA-Z0-9_/-]+$/).refine(v => v !== '/' && !v.split('/').includes('..'), 'Use a dedicated absolute container data path'),
  ports: z.array(portSchema).min(1).max(10),
  command: z.array(z.string().max(1000)).max(50).default([]),
  environment: z.record(z.string().regex(/^[A-Z_][A-Z0-9_]*$/), z.string().max(4096)).default({}),
  secretEnvironment: z.array(z.string().regex(/^[A-Z_][A-Z0-9_]*$/)).max(20).default([]),
}).strict().superRefine((v, ctx) => {
  for (const field of ['name', 'container']) {
    if (new Set(v.ports.map(p => p[field])).size !== v.ports.length) ctx.addIssue({ code: 'custom', message: `Duplicate port ${field}` });
  }
  if (v.secretEnvironment.some(k => Object.hasOwn(v.environment, k))) ctx.addIssue({ code: 'custom', message: 'Secret and ordinary environment names must differ' });
});

export const seeds = [
  { id: 'postgres', kind: 'postgres', description: 'Shared PostgreSQL 18; separate database and login per project.', image: 'postgres:18-alpine', dataPath: '/var/lib/postgresql', ports: [{ name: 'postgres', container: 5432, host: 5432 }], command: [], environment: {}, secretEnvironment: [] },
  { id: 'redis', kind: 'redis', description: 'Shared Redis 8 with AOF persistence and project ACL users/key prefixes.', image: 'redis:8-alpine', dataPath: '/data', ports: [{ name: 'redis', container: 6379, host: 6379 }], command: [], environment: {}, secretEnvironment: [] },
  { id: 'minio', kind: 'minio', description: 'Shared S3-compatible MinIO; project buckets and users. Archived upstream; built from the official security-fix source release.', image: 'development-mcp/minio:2025-10-15', dataPath: '/data', ports: [{ name: 's3', container: 9000, host: 9000 }, { name: 'console', container: 9001, host: 9001 }], command: [], environment: {}, secretEnvironment: [] },
];
