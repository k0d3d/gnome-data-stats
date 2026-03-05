import fs from 'fs';
import path from 'path';

const packageJsonPath = path.resolve('package.json');
const tauriConfPath = path.resolve('src-tauri/tauri.conf.json');
const cargoTomlPath = path.resolve('src-tauri/Cargo.toml');

// Read version from package.json (Source of Truth)
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

console.log(`Syncing version: ${version}`);

// 1. Update src-tauri/tauri.conf.json
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2));
console.log('✓ Updated src-tauri/tauri.conf.json');

// 2. Update src-tauri/Cargo.toml
let cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');
cargoToml = cargoToml.replace(/^version = ".*"/m, `version = "${version}"`);
fs.writeFileSync(cargoTomlPath, cargoToml);
console.log('✓ Updated src-tauri/Cargo.toml');

console.log('Version synchronization complete!');
