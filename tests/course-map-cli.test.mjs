import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
test('course map CLI renders and checks a synthetic workspace using the pinned OKF', t => {
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'course-map-cli-')));
  t.after(() => fs.rmSync(workspace, {recursive:true, force:true}));
  fs.cpSync(path.join(project,'tests/fixtures/workspace'),workspace,{recursive:true});
  const bin = path.join(workspace,'.local/bin');
  fs.mkdirSync(bin,{recursive:true});
  for (const [name,target] of [['okf','.local/bin/okf'],['learning-rules','target/debug/learning-rules']]) {
    fs.symlinkSync(path.join(project,target),path.join(bin,name));
  }
  const course = path.join(workspace,'courses/synthetic/course.md');
  fs.writeFileSync(course,fs.readFileSync(course,'utf8').replace('accepted_plan: /plans/map-v1','accepted_plan: /plans/map-v1\ncourse_map: {enabled: true}'));
  const original = fs.readFileSync(course,'utf8');
  const run = extra => {
    const p=spawnSync(process.execPath,[path.join(project,'scripts/update-course-map.mjs'),'--bundle','courses/synthetic',...extra],
      {cwd:workspace,env:{...process.env,LEARNING_WORKSPACE:workspace},encoding:'utf8'});
    assert.equal(p.status,0,p.stderr+p.stdout);
    return JSON.parse(p.stdout);
  };
  const rendered=run([]);
  assert.equal(rendered.updated,true);
  assert.ok(fs.existsSync(path.join(workspace,'courses/synthetic',rendered.snapshot)));
  assert.equal(run(['--check']).fresh,true);
  assert.equal(run([]).updated,false);
  assert.equal(fs.readFileSync(course,'utf8'),original);
});
