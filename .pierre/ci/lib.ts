import { run } from 'pierre';

export default async () => {
  const cwd = "lib";
  await run(`apt update -y && apt install -y build-essential`)
  await run(`cargo build`, { cwd });
  await run(`cargo test`, { cwd });
  await run(`cargo fmt --all -- --check`, { cwd });
  await run(`cargo clippy --all-targets -- -D warnings`, { cwd });
};
