import test from 'node:test';
import assert from 'node:assert/strict';
import {parseOperation, referenceId, chooseRun, eligiblePassage, retrieveBook} from '../scripts/lib/library-retrieval.mjs';
const source={id:'/sources/a',type:'Source',content_digest:'abc'};
const run={id:'/manifests/a/one',type:'IndexManifest',source_ref:'/sources/a',source_digest:'abc',manifest_status:'partial',page_count:3,indexed_ranges:['1:2'],no_text_ranges:['3'],failed_ranges:[],passage_refs:['/passages/a/one/p'],created_at:'2026-11-11T00:00:00Z'};
const passage={id:'/passages/a/one/p',type:'SourcePassage',source_ref:source.id,source_digest:'abc',manifest_ref:run.id,pdf_page_start:1,pdf_page_end:2,review_status:'unreviewed'};
const envelope=(records,query=false)=>JSON.stringify({kind:'operation-start',schema_version:1,stream_version:1})+'\n'+[...records,...(query?[{kind:'query-summary',scan_complete:true,partial:false,has_more:false,returned:records.length}]:[]),{kind:'operation-complete',schema_version:1,outcome:'ok',exit_code:0,completeness:{delivery:'complete',examination:'complete',matching:query?'complete':'not-applicable'}}].map(JSON.stringify).join('\n');
test('incomplete or bounded streams cannot establish absence',()=>{
 assert.throws(()=>parseOperation('',{query:true}));
 assert.throws(()=>parseOperation(envelope([]).replace('"delivery":"complete"','"delivery":"partial"')));
 assert.throws(()=>parseOperation(envelope([],true).replace('"scan_complete":true','"scan_complete":false'),{query:true}));
 assert.throws(()=>parseOperation(envelope([],true).replace('"has_more":false','"has_more":true'),{query:true}));
 assert.deepEqual(parseOperation(envelope([],true),{query:true}).filter(x=>x.id),[]);
});
test('usable partial runs survive newer failed/less complete runs and explicit selection survives sessions',()=>{
 const weaker={...run,id:'/manifests/a/two',indexed_ranges:['1'],created_at:'2026-11-12T00:00:00Z'};
 assert.equal(chooseRun(source,[weaker,run,{...run,manifest_status:'failed'}]).id,run.id);
 const selection={id:'/selections/a',source_ref:source.id,manifest_ref:weaker.id};
 assert.equal(chooseRun(source,[run,weaker],[selection]).id,weaker.id);
 assert.throws(()=>chooseRun({...source,content_digest:'changed'},[run],[selection]));
});
test('blocked, stale and wrong-run passages are excluded while unreviewed candidates stay readable',()=>{
 assert.equal(eligiblePassage(passage,source,run),true);
 assert.equal(eligiblePassage({...passage,source_digest:'old'},source,run),false);
 assert.equal(eligiblePassage({...passage,stale_after:'2000-01-01'},source,run),false);
 assert.equal(eligiblePassage(passage,source,run,[{id:'/reviews/a',passage_ref:passage.id,review_status:'blocked'}]),false);
 assert.equal(referenceId('/concepts/replication','../passages/a.md'),'/passages/a');
});
test('aliases navigate into opened selected-book passages without importing a second book',()=>{
 const other={...passage,id:'/passages/b/one/p',source_ref:'/sources/b'};
 const records=new Map([source,run,passage,other].map(r=>[r.id,r]));
 const opened=[];
 const cli=args=>{
  if(args[0]==='show') {opened.push(args[1]);return envelope(args.includes('--body')?[{kind:'body',id:args[1],body:'<!-- PDF page 1 -->\nOriginal synthetic source wording.'}]:[records.get(args[1])]);}
  const type=args[args.indexOf('--type')+1];
  return envelope(type==='IndexManifest'?[run]:type==='Concept'?[{id:'/concepts/replication',type:'Concept',aliases:['replica'],passage_refs:[passage.id,other.id]}]:[],true);
 };
 const result=retrieveBook(cli,source.id,'replica');
 assert.deepEqual(result.passages.map(p=>p.id),[passage.id]);
 assert.equal(result.passages[0].diagram_evidence,false);
 assert.equal(result.passages[0].response_check_required,true);
 assert.ok(opened.includes(passage.id));
 assert.equal(retrieveBook(args=>args[0]==='search'&&args.includes('Concept')?envelope([],true):cli(args),source.id,'absent').passages.length,0);
});
