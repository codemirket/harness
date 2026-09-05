const literal = value => `'${String(value).replaceAll("'", "''")}'`;
const identifier = value => `"${String(value).replaceAll('"', '""')}"`;

export async function provisionProject(manager, service, project) {
  const p = service.projects[project];
  const container = await manager.owned(service);
  if (service.template.kind === 'postgres') {
    const sql = `
SELECT 'CREATE ROLE ${identifier(p.username)} LOGIN PASSWORD ' || quote_literal(${literal(p.password)})
WHERE NOT EXISTS (SELECT FROM pg_roles WHERE rolname=${literal(p.username)})
\\gexec
SELECT 'CREATE DATABASE ${identifier(p.database)} OWNER ${identifier(p.username)}'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname=${literal(p.database)})
\\gexec
REVOKE CONNECT ON DATABASE ${identifier(p.database)} FROM PUBLIC;
GRANT CONNECT ON DATABASE ${identifier(p.database)} TO ${identifier(p.username)};
`;
    await manager.docker.run(['exec', '-i', container.Id, 'psql', '-X', '-v', 'ON_ERROR_STOP=1', '-U', 'devadmin', '-d', 'postgres'], { input: sql });
  } else if (service.template.kind === 'redis') {
    // Data commands only: no CONFIG, ACL, FLUSH*, KEYS, SCAN, SCRIPT, EVAL or administrative commands.
    const commands = ['+ping', '+echo', '+select', '+get', '+set', '+mget', '+mset', '+del', '+unlink', '+exists', '+expire', '+pexpire', '+ttl', '+pttl', '+persist', '+incr', '+incrby', '+decr', '+decrby', '+append', '+strlen', '+type', '+hget', '+hset', '+hgetall', '+hdel', '+hexists', '+hincrby', '+lpush', '+rpush', '+lpop', '+rpop', '+lrange', '+llen', '+sadd', '+srem', '+smembers', '+sismember', '+scard', '+zadd', '+zrem', '+zrange', '+zcard', '+zscore', '+multi', '+exec', '+discard', '+watch', '+unwatch', '+client|setinfo', '+client|setname'];
    await manager.redis(service, ['ACL', 'SETUSER', p.username, 'reset', 'on', `>${p.password}`, `~${p.prefix}*`, ...commands]);
    // Persist the ACL file with Redis itself, so users survive direct Docker restarts too.
    await manager.redis(service, ['ACL', 'SAVE']);
  } else if (service.template.kind === 'minio') {
    // A dedicated private config directory avoids credentials in mc command arguments.
    const config = { version: '10', aliases: { local: { url: 'http://127.0.0.1:9000', accessKey: 'devadmin', secretKey: service.secrets.password, api: 'S3v4', path: 'auto' } } };
    await manager.docker.run(['exec', '-i', container.Id, 'sh', '-c', 'umask 077; mkdir -p /tmp/development-mc; cat > /tmp/development-mc/config.json'], { input: JSON.stringify(config) });
    const mc = args => manager.docker.run(['exec', container.Id, 'mc', '--config-dir', '/tmp/development-mc', ...args]);
    await mc(['mb', '--ignore-existing', `local/${p.bucket}`]);
    const policy = { Version: '2012-10-17', Statement: [
      { Effect: 'Allow', Action: ['s3:ListBucket', 's3:GetBucketLocation', 's3:ListBucketMultipartUploads'], Resource: [`arn:aws:s3:::${p.bucket}`] },
      { Effect: 'Allow', Action: ['s3:GetObject', 's3:PutObject', 's3:DeleteObject', 's3:AbortMultipartUpload', 's3:ListMultipartUploadParts'], Resource: [`arn:aws:s3:::${p.bucket}/*`] },
    ] };
    await manager.docker.run(['exec', '-i', container.Id, 'sh', '-c', 'umask 077; cat > /tmp/development-mc/policy.json'], { input: JSON.stringify(policy) });
    await mc(['admin', 'policy', 'create', 'local', p.username, '/tmp/development-mc/policy.json']);
    // mc requires the password as an argument. The constant wrapper reads it from stdin
    // inside the container; the host Docker process never receives it as an argument.
    await manager.docker.run(['exec', '-i', container.Id, 'sh', '-c', 'IFS= read -r secret; exec mc --config-dir /tmp/development-mc admin user add local "$1" "$secret"', 'sh', p.username], { input: `${p.password}\n` });
    await mc(['admin', 'policy', 'attach', 'local', p.username, '--user', p.username]);
  }
}
