import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

try {
  // 1. Read version from package.json
  const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  const version = `v${packageJson.version}`;

  console.log(`🚀 Preparing release for ${version}...`);

  // 2. Sync versions across files
  console.log('🔄 Syncing versions...');
  execSync('npm run sync-version', { stdio: 'inherit' });

  // 3. Git Add and Commit
  console.log('📝 Committing version changes...');
  execSync('git add .', { stdio: 'inherit' });
  try {
    execSync(`git commit -m "chore: release ${version}"`, { stdio: 'inherit' });
  } catch (e) {
    console.log('⚠️ No changes to commit (version might already be synced).');
  }

  // 4. Create Tag
  console.log(`🏷️ Creating tag ${version}...`);
  try {
    execSync(`git tag -a ${version} -m "Release ${version}"`, { stdio: 'inherit' });
  } catch (e) {
    console.log(`⚠️ Tag ${version} already exists. Skipping tag creation.`);
  }

  // 5. Push to Master
  console.log('📤 Pushing to master and tags...');
  execSync('git push origin master --tags', { stdio: 'inherit' });

  console.log(`✅ Success! GitHub Actions will now build and draft your release.`);
} catch (error) {
  console.error('❌ Release failed:', error.message);
  process.exit(1);
}
