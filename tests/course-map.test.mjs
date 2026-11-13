import test from 'node:test';
import assert from 'node:assert/strict';
import {buildMap} from '../scripts/lib/course-map.mjs';
const r=(id,type,metadata={},body='')=>({id,metadata:{type,...metadata},body});
function fixture(){return {records:[
 r('/course','Course',{accepted_plan:'/plan',course_map:{enabled:true,title:'Synthetic course'}}),
 r('/plan','LearningPlan',{course:'/course',objectives:['/goal'],plan_status:'accepted',version:'1'}),
 r('/topic','Topic',{title:'Foundations'}),r('/goal','Objective',{topic:'/topic',title:'Goal'}),
 r('/lesson','Lesson',{title:'First lesson',objectives:['/goal'],readiness:'ready',estimated_seconds:60,cheat_sheet:'/sheets/one',exercise_refs:['/exercise']}),
 r('/session','Session',{current_lesson:'/lesson',started_at:'2026-01-01T00:00:00Z',session_status:'completed',completed_items:['one']}),
 r('/sheets/one','CheatSheet',{title:'Reference',lessons:['/lesson'],sections:['one','two'],revealed_sections:['one'],resource:'../references/private.html'},'## one\nCovered.\n\n## two\nHidden solution.\n'),
 ],history:{complete:true,attempts:[{id:'/attempt',exercise:'/exercise'}]},resolved_assessments:{'/attempt':{state:'resolved',outcome:'partial'}},history_digest:'synthetic'};}
const progress={objectives:{'/goal':{state:'practicing'}}};
test('completion does not imply demonstrated evidence; hidden references stay hidden',()=>{const m=buildMap(fixture(),'/course',progress,()=>true);const c=m.stages[0].cards[0];assert.equal(c.status,'covered');assert.deepEqual(c.evidence,['practicing']);assert.equal(c.judgments[0].outcome,'partial');assert.equal(c.refs[0].href,null);assert.equal(c.refs[0].visual,null);assert.equal(c.refs[0].sections.length,1);assert(!JSON.stringify(m).includes('Hidden solution'));});
test('preview remains a preview rather than completed study',()=>{const s=fixture();Object.assign(s.records[5].metadata,{session_status:'paused',current_item:'preview-only-return',completed_items:['preview-one']});assert.equal(buildMap(s,'/course',progress,()=>true).stages[0].cards[0].status,'preview');});
test('current session follows the latest saved active cursor',()=>{const s=fixture();s.records[5].metadata.session_status='active';const m=buildMap(s,'/course',progress,()=>true);assert.equal(m.current.id,'/lesson');assert.equal(m.stats.covered,0);});
test('initial plan renders before any lessons exist',()=>{const s=fixture();s.records=s.records.filter(r=>!['Lesson','Session','CheatSheet'].includes(r.metadata.type));s.history.attempts=[];const m=buildMap(s,'/course',progress,()=>true);assert.equal(m.stages.length,1);assert.equal(m.stages[0].cards.length,0);assert.equal(m.current,null);});
test('out-of-bundle, missing and executable URL references are not exposed',()=>{for(const resource of ['../../../elsewhere.html','javascript:alert(1)','https://example.org/tracker.html']){const s=fixture();Object.assign(s.records[6].metadata,{revealed_sections:['one','two'],resource});assert.equal(buildMap(s,'/course',progress,()=>true).stages[0].cards[0].refs[0].visual,null);}const s=fixture();Object.assign(s.records[6].metadata,{revealed_sections:['one','two']});assert.equal(buildMap(s,'/course',progress,()=>false).stages[0].cards[0].refs[0].visual,null);});
test('complete revealed reference has a portable artifact link',()=>{const s=fixture();s.records[6].metadata.revealed_sections=['one','two'];assert.equal(buildMap(s,'/course',progress,()=>true).stages[0].cards[0].refs[0].visual,'../../references/private.html');});
test('incomplete inventory or foreign plan cannot generate a map',()=>{const s=fixture();s.history.complete=false;assert.throws(()=>buildMap(s,'/course',progress,()=>true));s.history.complete=true;s.records[1].metadata.course='/another';assert.throws(()=>buildMap(s,'/course',progress,()=>true));});
