#!/usr/bin/env node
const { spawn } = require('node:child_process');
const { optionalDependencies } = require('../package.json');

const platform = `${process.platform}-${process.arch}`;
const packageName = `burstq-${platform}`;
if (!Object.hasOwn(optionalDependencies, packageName)) {
  const supported = Object.keys(optionalDependencies).map((name) => name.slice('burstq-'.length));
  console.error(`burstq: unsupported platform ${platform}. Supported: ${supported.join(', ')}.`);
  process.exit(1);
}

let binary;
try {
  binary = require.resolve(`${packageName}/bin/burstq`);
} catch {
  console.error(`burstq: missing ${packageName}. Reinstall burstq with optional dependencies enabled (npm install -g burstq --include=optional).`);
  process.exit(1);
}

const child = spawn(binary, process.argv.slice(2), { stdio: 'inherit' });
const handlers = new Map();
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) {
  const handler = () => child.kill(signal);
  handlers.set(signal, handler);
  process.on(signal, handler);
}
child.on('error', (error) => {
  console.error(`burstq: could not start ${packageName}: ${error.message}`);
  process.exitCode = 1;
});
child.on('exit', (code, signal) => {
  for (const [name, handler] of handlers) process.removeListener(name, handler);
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = code;
});
