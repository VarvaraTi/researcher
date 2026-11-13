#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {buildMap} from './lib/course-map.mjs';
const pluginRoot=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const root=path.resolve(process.env.LEARNING_WORKSPACE||process.cwd());
function run(exe,args,input){const r=spawnSync(exe,args,{cwd:root,input,encoding:'utf8',maxBuffer:32*1024*1024});if(r.error||r.status!==0)throw new Error(r.error?.message||r.stderr||r.stdout);return r.stdout;}
const okf=path.join(root,'.local/bin/okf'),installedRules=path.join(root,'.local/bin/learning-rules');
const rules=fs.existsSync(installedRules)?installedRules:path.join(root,'target/debug/learning-rules');
function rule(command,data){return JSON.parse(run(rules,[command],JSON.stringify({schema_version:1,data}))).result;}
function cli(args){const rows=run(okf,[...args,'--json']).trim().split('\n').map(JSON.parse);if(rows[0]?.kind!=='operation-start'||rows.at(-1)?.kind!=='operation-complete'||rows.at(-1).outcome!=='ok')throw new Error('Incomplete OKF operation');return rows;}
const hash=x=>crypto.createHash('sha256').update(x).digest('hex');
try{
 const args=process.argv.slice(2);if(args.includes('--help')){console.log('Usage: node scripts/update-course-map.mjs --bundle courses/COURSE [--course /course] [--check]\nFull courses only. Requires Node 20+, local OKF 0.6.0 and learning-rules.');process.exit(0);}
 const at=flag=>args.indexOf(flag);const value=(flag,fallback)=>at(flag)>=0?args[at(flag)+1]:fallback;
 if(args.some((x,i)=>!['--bundle','--course','--check'].includes(x)&&!['--bundle','--course'].includes(args[i-1])))throw new Error('Unknown argument');
 if(!value('--bundle'))throw new Error('--bundle is required');
 const bundle=fs.realpathSync(path.resolve(root,value('--bundle'))),courseId=value('--course','/course');
 if(!bundle.startsWith(path.join(root,'courses')+path.sep))throw new Error('Map must belong to a course in this workspace.');
 if(fs.existsSync(path.join(bundle,'.okf-transaction')))throw new Error('Recover the pending OKF transaction first.');
 if(run(okf,['--version']).trim()!=='okf 0.6.0')throw new Error('Expected pinned OKF 0.6.0');
 const guidance=cli(['bundle','context',bundle,'--operation','search','--operation','show','--operation','artifact put']);
 // A standalone helper cannot interpret newly added natural-language guidance.
 if(guidance.some(x=>x.kind==='task-guidance'))throw new Error('Bundle has task guidance: have the coordinator review it before using this helper.');
 const sourceRows=cli(['search',bundle,'--full-scan']);
 const hasSharedBook=sourceRows.some(r=>['shared_source_refs','shared_passage_refs','shared_concept_refs'].some(field=>r[field]?.length));
 const context={okf,bundle,scope_bundles:['learning.learner',...(hasSharedBook?['learning.library']:[])]};
 const state=rule('inspect-state',context);
 if(state.records.filter(r=>r.metadata.type==='Course').length!==1||state.records.some(r=>r.metadata.type==='Study'))throw new Error('This helper currently supports one full Course per bundle.');
 const progress=rule('compute-progress',{history:state.history,objectives:state.objectives,required_items:[],completed_items:[],skipped_items:[]});
 const exists=target=>{try{const real=fs.realpathSync(path.join(bundle,target));return real.startsWith(bundle+path.sep)&&fs.statSync(real).isFile();}catch{return false;}};
 const model=buildMap(state,courseId,progress,exists),template=fs.readFileSync(path.join(pluginRoot,'templates/course-map.html'),'utf8');
 const digest=hash(JSON.stringify({model,template}));const resource=`references/visualizations/course-map-${digest.slice(0,20)}.html`;
 const entry='<!-- learning-course-map-entry:v1 -->\n<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Course map</title><style>html,body,iframe{margin:0;width:100%;height:100%;border:0}body{background:#0c1220}</style><iframe title="Course learning map" src="'+resource+'"></iframe><noscript><a href="'+resource+'">Open course map</a></noscript></html>\n';
 const entryPath=path.join(bundle,'course-map.html');
 if(fs.existsSync(entryPath)&&!fs.readFileSync(entryPath,'utf8').startsWith('<!-- learning-course-map-entry:v1 -->'))throw new Error('Refusing to replace an unrelated course-map.html');
 const fresh=exists(resource)&&fs.existsSync(entryPath)&&fs.readFileSync(entryPath,'utf8')===entry;
 if(args.includes('--check')){console.log(JSON.stringify({fresh,entry:entryPath,snapshot:resource}));process.exit(fresh?0:1);}
 if(fresh){console.log(JSON.stringify({updated:false,entry:entryPath,snapshot:resource}));process.exit(0);}
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'learning-map-'));
 try{
  const data=JSON.stringify({...model,generatedAt:new Date().toISOString(),snapshotDigest:digest}).replaceAll('<','\\u003c');
  const html=template.replace('/*COURSE_MAP_DATA*/',data);
  fs.writeFileSync(path.join(temp,'snapshot.html'),html);fs.writeFileSync(path.join(temp,'entry.html'),entry);
  // Refuse activation if any learning record changed while rendering.
  if(hash(JSON.stringify(rule('inspect-state',context).records))!==hash(JSON.stringify(state.records)))throw new Error('Learning records changed; rerun to refresh from the new state.');
  if(!exists(resource))cli(['artifact','put',resource,'@'+path.join(temp,'snapshot.html'),bundle]);
  cli(['artifact','put','course-map.html','@'+path.join(temp,'entry.html'),bundle,...(fs.existsSync(entryPath)?['--replace']:[])]);
  console.log(JSON.stringify({updated:true,entry:entryPath,snapshot:resource,lessons:model.stats.prepared}));
 }finally{fs.rmSync(temp,{recursive:true,force:true});}
}catch(error){console.error('course-map: '+error.message);process.exitCode=1;}
