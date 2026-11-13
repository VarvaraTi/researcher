import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {buildChangeset} from '../scripts/lib/book-ingestion.mjs';
import {retrieveBook, parseOperation} from '../scripts/lib/library-retrieval.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const okf = path.join(root, '.local/bin/okf');
const script = process.env.LIBRARY_RETRIEVAL_SCRIPT || path.join(root, 'scripts/retrieve-book.mjs');
const sha = bytes => 'sha256:' + createHash('sha256').update(bytes).digest('hex');
function invoke(cmd, args, cwd, input) {
  const p = spawnSync(cmd, args, {cwd, input, encoding:'utf8', maxBuffer:64*1024*1024});
  assert.equal(p.status, 0, `${cmd} ${args.join(' ')}\n${p.stderr}\n${p.stdout}`);
  return p.stdout;
}
function put(root, relative, content) {
  const file = path.join(root, relative);
  fs.mkdirSync(path.dirname(file), {recursive:true});
  fs.writeFileSync(file, content);
}
function fixture(t) {
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(),'library-integration-'));
  t.after(() => fs.rmSync(workspace,{recursive:true,force:true}));
  const bundle = path.join(workspace,'courses/library');
  invoke(okf,['init',bundle,'--title','Synthetic library'],workspace);
  invoke(okf,['ontology','apply',bundle,'--from',path.join(root,'schemas/library-ontology.yaml')],workspace);
  fs.mkdirSync(path.join(workspace,'.local/bin'),{recursive:true});
  fs.symlinkSync(okf,path.join(workspace,'.local/bin/okf'));
  put(workspace,'okf-catalog.yaml',`catalog_version: 2\nbundles:\n  learning.library:\n    location: {type: directory, path: courses/library}\n`);
  const extraction = text => ({extractor:'synthetic',extractor_version:'1',page_count:2,pages:[{page:1,state:'text',text},{page:2,state:'no_text'}]});
  const operations = [];
  for (const [id, text] of [['alpha','# System\n\nA cache accelerates reads.\n'],['beta','# Music\n\nA cache stores musical fragments.\n']]) {
    const bytes = Buffer.from('synthetic source '+id);
    const built = buildChangeset({sourceId:id,runId:'run-one',title:`Book ${id}`,edition:'First',language:'en'},extraction(text),sha(bytes),'2026-01-01T00:00:00Z');
    put(bundle,built.artifact,bytes);
    operations.push(...built.changeset.operations);
  }
  operations.push({op:'create',path:'concepts/cache.md',content:`---\ntype: Concept\nschema_version: 1\ntitle: Shared cache\naliases: ["buffering"]\npassage_refs: ["/passages/alpha/run-one/p1-p1-part0001", "/passages/beta/run-one/p1-p1-part0001"]\n---\nSynthetic navigation.\n`});
  const changeset = {version:1,operations};
  const plan = parseOperation(invoke(okf,['changeset','plan',bundle,'--from',JSON.stringify(changeset),'--json'],workspace));
  const digest = plan.find(row=>row.kind === 'changeset-summary')?.base_digest;
  assert.ok(digest, 'OKF plan supplies a baseline digest');
  const publish = {schema_version:1,data:{mode:'library',context:{okf,bundle},changeset,expected_digest:digest}};
  const published = JSON.parse(invoke(path.join(root,'target/debug/learning-rules'),['publish'],workspace,JSON.stringify(publish)));
  assert.equal(published.result.publication,'applied');
  const cli = args => invoke(okf,[...args,'--catalog',path.join(workspace,'okf-catalog.yaml'),'--bundle-id','learning.library'],workspace);
  return {workspace,bundle,cli};
}

test('pinned OKF opens book-scoped passages and concept aliases with complete query evidence', t => {
  const {cli} = fixture(t);
  const alpha = retrieveBook(cli,'/sources/alpha','cache');
  assert.equal(alpha.manifest.manifest_status,'partial');
  assert.deepEqual(alpha.manifest.no_text_ranges,['2']);
  assert.equal(alpha.passages.length,1);
  assert.match(alpha.passages[0].body,/accelerates reads/);
  assert.equal(alpha.passages[0].response_check_required,true);
  assert.equal(alpha.passages[0].diagram_evidence,false);
  const alias = retrieveBook(cli,'/sources/alpha','buffering');
  assert.deepEqual(alias.passages.map(p=>p.id),alpha.passages.map(p=>p.id));
  const beta = retrieveBook(cli,'/sources/beta','cache');
  assert.equal(beta.passages.length,1);
  assert.match(beta.passages[0].body,/musical fragments/);
  assert.equal(retrieveBook(cli,'/sources/alpha','nonexistentterm').passages.length,0);
  const incomplete = args => cli(args).replace('"scan_complete":true','"scan_complete":false');
  assert.throws(()=>retrieveBook(incomplete,'/sources/alpha','cache'),/Incomplete OKF query/);
});

test('standalone retrieval requires explicit comparison and detects retained source drift', t => {
  const {workspace,bundle} = fixture(t);
  const args = ['--workspace',workspace,'--text','cache','--source','/sources/alpha','--source','/sources/beta'];
  const noCompare = spawnSync(process.execPath,[script,...args],{encoding:'utf8'});
  assert.notEqual(noCompare.status,0);
  assert.match(noCompare.stderr,/Multiple books/);
  const result = JSON.parse(invoke(process.execPath,[script,...args,'--compare'],workspace));
  assert.equal(result.comparison,true);
  assert.deepEqual(result.results.map(r=>r.source.id),['/sources/alpha','/sources/beta']);
  assert.match(result.results[0].passages[0].body,/accelerates reads/);
  assert.match(result.results[1].passages[0].body,/musical fragments/);
  const alpha = result.results[0].source;
  fs.writeFileSync(path.join(bundle,alpha.location),'changed bytes');
  const drift = spawnSync(process.execPath,[script,'--workspace',workspace,'--text','cache','--source','/sources/alpha'],{encoding:'utf8'});
  assert.notEqual(drift.status,0);
  assert.match(drift.stderr,/drift|stale/i);
});

test('selected course determines default book and multi-book courses do not compare automatically', t => {
  const {workspace} = fixture(t);
  const course = path.join(workspace,'courses/course');
  invoke(okf,['init',course,'--title','Synthetic course'],workspace);
  put(course,'course.md',`---\ntype: Course\ntitle: Synthetic course\nshared_source_refs: ["../library/sources/alpha.md"]\n---\nSynthetic course.\n`);
  fs.appendFileSync(path.join(workspace,'okf-catalog.yaml'),'  learning.course:\n    location: {type: directory, path: courses/course}\n');
  put(workspace,'learning.local.json',JSON.stringify({selected_course:'learning.course'}));
  const args = ['--workspace',workspace,'--text','cache'];
  const result = JSON.parse(invoke(process.execPath,[script,...args],workspace));
  assert.deepEqual(result.scope,['/sources/alpha']);
  assert.equal(result.comparison,false);
  assert.match(result.results[0].passages[0].body,/accelerates reads/);
  put(course,'course.md',`---\ntype: Course\ntitle: Synthetic course\nshared_source_refs: ["../library/sources/alpha.md", "../library/sources/beta.md"]\n---\nSynthetic course.\n`);
  const ambiguous = spawnSync(process.execPath,[script,...args],{encoding:'utf8'});
  assert.notEqual(ambiguous.status,0);
  assert.match(ambiguous.stderr,/Multiple books/);
  const comparison = JSON.parse(invoke(process.execPath,[script,...args,'--compare'],workspace));
  assert.deepEqual(comparison.scope,['/sources/alpha','/sources/beta']);
});

test('blocked review and stale passage are excluded by actual OKF searches', t => {
  const {bundle,cli} = fixture(t);
  put(bundle,'reviews/blocked.md',`---\ntype: PassageReview\nschema_version: 1\ntitle: Blocked synthetic passage\npassage_ref: /passages/alpha/run-one/p1-p1-part0001\nreview_status: blocked\nreviewed_at: "2026-01-01T00:00:00Z"\nreviewer: Synthetic reviewer\nevidence: Synthetic block\n---\nBlocked.\n`);
  const blocked = retrieveBook(cli,'/sources/alpha','cache');
  assert.equal(blocked.passages.length,0);
  assert.deepEqual(blocked.excluded,['/passages/alpha/run-one/p1-p1-part0001']);
  const beta = path.join(bundle,'passages/beta/run-one/p1-p1-part0001.md');
  fs.writeFileSync(beta,fs.readFileSync(beta,'utf8').replace('schema_version: 1','schema_version: 1\nstale_after: "2000-01-01"'));
  const stale = retrieveBook(cli,'/sources/beta','cache');
  assert.equal(stale.passages.length,0);
  assert.deepEqual(stale.excluded,['/passages/beta/run-one/p1-p1-part0001']);
});

test('course publication requires library scope and rejects a shared target of the wrong type', t => {
  const {workspace} = fixture(t);
  const original = path.join(root,'tests/fixtures/workspace');
  fs.cpSync(path.join(original,'learner'),path.join(workspace,'learner'),{recursive:true});
  fs.cpSync(path.join(original,'courses/synthetic'),path.join(workspace,'courses/synthetic'),{recursive:true});
  const course = path.join(workspace,'courses/synthetic');
  invoke(okf,['ontology','apply',course,'--from',path.join(root,'schemas/course-ontology.yaml')],workspace);
  fs.appendFileSync(path.join(workspace,'okf-catalog.yaml'),'  learning.learner:\n    location: {type: directory, path: learner}\n  learning.synthetic:\n    location: {type: directory, path: courses/synthetic}\n');
  function attempt(ref, scopes) {
    const changeset = {version:1,operations:[{op:'edit',concept:'course',patch:[{op:'add',path:'/shared_source_refs',value:[ref]}]}]};
    const plan = parseOperation(invoke(okf,['changeset','plan',course,'--from',JSON.stringify(changeset),'--json'],workspace));
    const expected_digest = plan.find(row=>row.kind === 'changeset-summary').base_digest;
    const request = {schema_version:1,data:{context:{okf,bundle:course,scope_bundles:scopes},changeset,expected_digest}};
    return spawnSync(path.join(root,'target/debug/learning-rules'),['publish'],{cwd:workspace,input:JSON.stringify(request),encoding:'utf8'});
  }
  const missingScope = attempt('../library/sources/alpha.md',['learning.learner']);
  assert.notEqual(missingScope.status,0,missingScope.stdout);
  const wrongType = attempt('../library/concepts/cache.md',['learning.learner','learning.library']);
  assert.notEqual(wrongType.status,0,wrongType.stdout);
  const valid = attempt('../library/sources/alpha.md',['learning.learner','learning.library']);
  assert.equal(valid.status,0,valid.stdout + valid.stderr);
  assert.equal(JSON.parse(valid.stdout).result.publication,'applied');
});

test('library publication rejects invalid enums, missing fields, immutable edits and inconsistent runs', t => {
  const {workspace,bundle} = fixture(t);
  const attempt = (operations) => {
    const changeset = {version:1,operations};
    const plan = spawnSync(okf,['changeset','plan',bundle,'--from',JSON.stringify(changeset),'--json'],{cwd:workspace,encoding:'utf8'});
    if (plan.status !== 0) return {status:plan.status,output:plan.stderr+plan.stdout};
    const expected_digest = parseOperation(plan.stdout).find(row=>row.kind==='changeset-summary').base_digest;
    const request = {schema_version:1,data:{mode:'library',context:{okf,bundle},changeset,expected_digest}};
    const published = spawnSync(path.join(root,'target/debug/learning-rules'),['publish'],{cwd:workspace,input:JSON.stringify(request),encoding:'utf8'});
    return {status:published.status,output:published.stdout+published.stderr};
  };
  const edit = patch => [{op:'edit',concept:'passages/alpha/run-one/p1-p1-part0001',patch}];
  const bytes = Buffer.from('synthetic third source');
  const built = buildChangeset({sourceId:'gamma',runId:'run-one',title:'Book gamma',edition:'First',language:'en'}, {extractor:'synthetic',extractor_version:'1',page_count:1,pages:[{page:1,state:'text',text:'Fresh content.'}]}, sha(bytes), '2026-01-01T00:00:00Z');
  put(bundle,built.artifact,bytes);
  const inconsistent = (index, from, to) => built.changeset.operations.map((op,i)=>i===index?{...op,content:op.content.replace(from,to)}:op);
  for (const [label,operations] of [
    ['invalid enum',edit([{op:'replace',path:'/review_status',value:'invented'}])],
    ['missing required field',edit([{op:'remove',path:'/source_ref'}])],
    ['immutable body',[{op:'edit',concept:'passages/alpha/run-one/p1-p1-part0001',body:'Changed source text.'}]],
    ['source digest mismatch',inconsistent(1,sha(bytes),'sha256:wrong')],
    ['coverage mismatch',inconsistent(2,'indexed_ranges: [\"1\"]','indexed_ranges: []')]
  ]) {
    const result=attempt(operations);
    assert.notEqual(result.status,0,`${label} unexpectedly accepted: ${result.output}`);
  }
});

test('a guarded new run preserves old passages and an explicit selection survives a new process', t => {
  const {workspace,bundle,cli} = fixture(t);
  const digest = sha(Buffer.from('synthetic source alpha'));
  const built = buildChangeset({sourceId:'alpha',runId:'run-two',title:'Book alpha',edition:'First',language:'en',existingSource:true},
    {extractor:'synthetic',extractor_version:'1',page_count:2,pages:[{page:1,state:'text',text:'Alternate cache wording.'},{page:2,state:'no_text'}]},digest,'2026-02-01T00:00:00Z');
  const selection = {op:'create',path:'selections/alpha.md',content:'---\ntype: SourceSelection\nschema_version: 1\ntitle: Explicit synthetic selection\nsource_ref: /sources/alpha\nmanifest_ref: /manifests/alpha/run-two\n---\nExplicit re-index choice.\n'};
  const changeset = {version:1,operations:[...built.changeset.operations,selection]};
  const plan = parseOperation(invoke(okf,['changeset','plan',bundle,'--from',JSON.stringify(changeset),'--json'],workspace));
  const request = {schema_version:1,data:{mode:'library',context:{okf,bundle},changeset,expected_digest:plan.find(r=>r.kind==='changeset-summary').base_digest}};
  const result=JSON.parse(invoke(path.join(root,'target/debug/learning-rules'),['publish'],workspace,JSON.stringify(request)));
  assert.equal(result.result.publication,'applied');
  assert.ok(fs.existsSync(path.join(bundle,'passages/alpha/run-one/p1-p1-part0001.md')));
  const fresh = JSON.parse(invoke(process.execPath,[script,'--workspace',workspace,'--text','cache','--source','/sources/alpha'],workspace));
  assert.equal(fresh.results[0].manifest.id,'/manifests/alpha/run-two');
  assert.match(fresh.results[0].passages[0].body,/Alternate cache wording/);
  const retry = spawnSync(path.join(root,'target/debug/learning-rules'),['publish'],{cwd:workspace,input:JSON.stringify(request),encoding:'utf8'});
  assert.notEqual(retry.status,0);
  assert.equal(retrieveBook(cli,'/sources/alpha','cache').manifest.id,'/manifests/alpha/run-two');
  // Simulate a corrupted saved choice pointing at a failed extraction: reads fail closed.
  const failed = buildChangeset({sourceId:'alpha',runId:'run-failed',title:'Book alpha',edition:'First',language:'en',existingSource:true}, {extractor:'synthetic',extractor_version:'1',page_count:null,pages:[],document_failure:'Synthetic extraction failure'},digest,'2026-03-01T00:00:00Z');
  for (const op of failed.changeset.operations) put(bundle,op.path,op.content);
  const selectionPath = path.join(bundle,'selections/alpha.md');
  fs.writeFileSync(selectionPath,fs.readFileSync(selectionPath,'utf8').replace('/manifests/alpha/run-two','/manifests/alpha/run-failed'));
  assert.throws(()=>retrieveBook(cli,'/sources/alpha','cache'),/Saved run is stale, failed or unavailable/);
});
