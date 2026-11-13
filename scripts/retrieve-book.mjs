#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {parseOperation, retrieveBook} from './lib/library-retrieval.mjs';

try {
  const args = process.argv.slice(2), options = {}, sources = [];
  for (let i=0; i<args.length; i++) {
    if (args[i] === '--help') {
      console.log('Usage: node scripts/retrieve-book.mjs --text QUERY [--source /sources/ID ... --compare] [--workspace PATH]\nWithout --source, use the selected course record shared_source_refs. Read-only; requires Node and pinned OKF 0.6.0.');
      process.exit(0);
    }
    if (args[i] === '--compare') options.compare = true;
    else if (['--text','--source','--workspace'].includes(args[i]) && args[i+1] && !args[i+1].startsWith('--')) {
      const key = args[i++];
      if (key === '--source') sources.push(args[i]); else options[key.slice(2)] = args[i];
    } else throw new Error('Unknown or missing argument: '+args[i]);
  }
  if (!options.text?.trim()) throw new Error('--text is required.');
  const root = fs.realpathSync(options.workspace || process.env.LEARNING_WORKSPACE || process.cwd());
  const executable = path.join(root,'.local/bin/okf'), catalog = path.join(root,'okf-catalog.yaml');
  const run = args => {
    const result = spawnSync(executable,args,{cwd:root,encoding:'utf8',maxBuffer:64*1024*1024});
    if (result.error || result.status !== 0) throw new Error(result.error?.message || result.stderr || result.stdout);
    return result.stdout;
  };
  if (run(['version']).trim() !== 'okf 0.6.0 (OKF spec 0.2)') throw new Error('Expected pinned OKF 0.6.0.');
  const library = args => run([...args,'--catalog',catalog,'--bundle-id','learning.library']);
  const context = parseOperation(library(['bundle','context','--operation','search','--operation','show','--operation','stale','--json']));
  if (context.some(r => r.kind === 'task-guidance')) throw new Error('Coordinator must review bundle guidance before using this standalone helper.');
  if (!sources.length) {
    const config = JSON.parse(fs.readFileSync(path.join(root,'learning.local.json'),'utf8'));
    if (!config.selected_course) throw new Error('Select a course or name a book explicitly.');
    const courseContext = parseOperation(run(['bundle','context','--catalog',catalog,'--bundle-id',config.selected_course,'--scope-bundle','learning.library','--operation','show','--operation','resolve','--json']));
    if (courseContext.some(r => r.kind === 'task-guidance')) throw new Error('Coordinator must review selected-course guidance first.');
    const courseRows = parseOperation(run(['show',config.selected_course_record || '/course','--catalog',catalog,'--bundle-id',config.selected_course,'--json']));
    const course = courseRows.find(r => r.type === 'Course' || r.type === 'Study');
    if (!course) throw new Error('Selected course/study is unavailable.');
    for (const ref of course.shared_source_refs || []) {
      // Resolve authored links through OKF rather than guessing their bundle target.
      const resolved = parseOperation(run(['resolve',ref,'--from',course.id,'--catalog',catalog,'--bundle-id',config.selected_course,'--scope-bundle','learning.library','--json']));
      const target = resolved.find(r => r.kind === 'bundle-edge' && r.status === 'resolved')?.target;
      if (!target || target.bundle !== 'learning.library') throw new Error('Shared source must resolve into learning.library; select --source explicitly.');
      sources.push(target.id);
    }
  }
  if (!sources.length) throw new Error('No selected shared book; prepare/link the selected edition first.');
  if (sources.length > 1 && !options.compare) throw new Error('Multiple books require clarification or an explicit --compare.');
  // Fingerprint findings are checked before any opened content is returned.
  const health = parseOperation(library(['stale','--json']));
  const results = sources.map(id => retrieveBook(library,id,options.text));
  const registrations = parseOperation(run(['catalog','--catalog',catalog,'--json']));
  const registration = registrations.find(r => r.kind === 'bundle-registration' && r.id === 'learning.library');
  if (!registration?.root) throw new Error('Library registration unavailable.');
  const libraryRoot = fs.realpathSync(registration.root);
  for (const result of results) {
    const retained = fs.realpathSync(path.resolve(libraryRoot, result.source.location));
    if (!retained.startsWith(libraryRoot + path.sep)) throw new Error('Source artifact escapes library.');
    const digest = 'sha256:' + createHash('sha256').update(fs.readFileSync(retained)).digest('hex');
    if (digest !== result.source.content_digest) throw new Error('Retained source digest drift; decide the update explicitly.');
    const ids = new Set([result.source.id,result.manifest.id,...result.passages.map(p=>p.id)]);
    if (health.some(r => ids.has(r.id || r.concept) && ['missing','unavailable','unsupported','drifted','revision-changed'].includes(r.state || r.status)))
      throw new Error('Relevant source fingerprint is stale/unavailable; review it before grounding a response.');
  }
  console.log(JSON.stringify({scope:sources,comparison:!!options.compare,results},null,2));
} catch (error) {
  console.error('retrieve-book: '+error.message);
  process.exitCode = 1;
}
