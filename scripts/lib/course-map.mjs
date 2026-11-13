import path from 'node:path';

export function buildMap(state, courseId, progress, exists) {
  if (!state.history?.complete) throw new Error('A complete history is required.');
  const records = new Map(state.records.map(r => [r.id, r]));
  const course = records.get(courseId);
  if (course?.metadata.type !== 'Course') throw new Error('Select a full-course Course record.');
  const config = course.metadata.course_map;
  if (!config?.enabled) throw new Error('This course has not enabled its visual map.');
  const plan = records.get(course.metadata.accepted_plan || config.proposed_plan);
  if (plan?.metadata.type !== 'LearningPlan' || plan.metadata.course !== courseId) throw new Error('No selected course plan.');
  const meta = id => records.get(id)?.metadata;
  const local = (owner, resource) => {
    if (typeof resource !== 'string' || /^(?:[a-z]+:|\/\/)/i.test(resource)) return null;
    const target = resource.startsWith('/') ? resource.slice(1) : path.posix.normalize(path.posix.join(path.posix.dirname(owner.slice(1)), resource));
    if (target === '..' || target.startsWith('../') || !exists(target)) return null;
    return '../../' + target.split('/').map(encodeURIComponent).join('/');
  };
  const recordLink = id => local('/', id + '.md');
  const topicIds = [...new Set(plan.metadata.objectives.map(id => meta(id)?.topic).filter(Boolean))];
  const topics = new Set(topicIds);
  const lessons = state.records.filter(r => r.metadata.type === 'Lesson' && r.metadata.objectives.some(id => topics.has(meta(id)?.topic)));
  const sessions = state.records.filter(r => r.metadata.type === 'Session' && lessons.some(l => l.id === r.metadata.current_lesson));
  const active = sessions.filter(r => r.metadata.session_status === 'active').sort((a,b) => b.metadata.started_at.localeCompare(a.metadata.started_at));
  const currentLesson = active[0]?.metadata.current_lesson;
  const topicOverrides = config.topics || {};
  function reference(sheet) {
    const m = sheet.metadata, revealed = new Set(m.revealed_sections || []);
    if (!revealed.size) return null;
    // Publish only explicitly revealed sections, never a hidden answer or future section.
    const sections = [...sheet.body.matchAll(/^## (.+)\r?\n([\s\S]*?)(?=^## |$(?![\s\S]))/gm)]
      .filter(x => revealed.has(x[1].trim())).map(x => ({title:x[1].trim().replaceAll('-',' '),text:x[2].trim()}));
    const fullyRevealed = (m.sections || []).every(id => revealed.has(id));
    return {id:sheet.id,title:m.title,sections,href:fullyRevealed?recordLink(sheet.id):null,
      visual:fullyRevealed && m.resource?.endsWith('.html') ? local(sheet.id,m.resource) : null};
  }
  const cards = lessons.map(l => {
    const m=l.metadata, runs=sessions.filter(r=>r.metadata.current_lesson===l.id).sort((a,b)=>b.metadata.started_at.localeCompare(a.metadata.started_at));
    const recent=runs[0]?.metadata;
    const preview=!!recent?.current_item?.startsWith('preview-') || (recent?.completed_items?.length>0 && recent.completed_items.every(x=>x.startsWith('preview-')));
    let status = m.readiness!=='ready'?'unavailable':l.id===currentLesson?'current':preview?'preview':recent?.session_status==='completed'?'covered':recent?.session_status==='paused'?'paused':recent?'in-progress':'prepared';
    const attempts=state.history.attempts.filter(a=>(m.exercise_refs||[]).includes(a.exercise));
    const ids=new Set(attempts.map(a=>a.id));
    const judgments=Object.entries(state.resolved_assessments||{}).filter(([id])=>ids.has(id)).map(([id,v])=>({id,...v}));
    const evidence=m.objectives.map(id=>progress.objectives[id]?.state).filter(Boolean);
    const refs=state.records.filter(r=>r.metadata.type==='CheatSheet' && ((r.metadata.lessons||[]).includes(l.id)||r.id===m.cheat_sheet)).map(reference).filter(Boolean);
    return {id:l.id,title:m.title,search:[m.title,...m.objectives.map(id=>meta(id)?.title||'')].join(' '),topic:meta(m.objectives[0])?.topic,status,minutes:Math.round(m.estimated_seconds/60),version:m.version,
      attempts:attempts.length,judgments,evidence,refs,
      lesson:recordLink(l.id),objectives:m.objectives.map(id=>({id,title:meta(id)?.title,state:progress.objectives[id]?.state||'not_started'})),
      order:config.lesson_order?.indexOf(l.id)??-1,started:runs.at(-1)?.metadata.started_at||'9999'};
  }).sort((a,b)=> (a.order>=0?a.order:999)-(b.order>=0?b.order:999)||a.started.localeCompare(b.started)||a.id.localeCompare(b.id));
  const stages=topicIds.map((id,i)=>({id,number:i+1,title:topicOverrides[id]?.title||meta(id)?.title||id,
    subtitle:topicOverrides[id]?.subtitle||'',scope:topicOverrides[id]?.scope||[],href:recordLink(id),
    prerequisites:meta(id)?.prerequisites||[],cards:cards.filter(c=>c.topic===id)}));
  return {title:config.title||course.metadata.title,subtitle:config.subtitle||'',plan:{title:plan.metadata.title,status:plan.metadata.plan_status,version:plan.metadata.version,href:recordLink(plan.id)},
    stages,current:cards.find(c=>c.id===currentLesson)||null,activeCount:active.length,
    stats:{prepared:cards.length,covered:cards.filter(c=>c.status==='covered').length,preview:cards.filter(c=>c.status==='preview').length,attempts:state.history.attempts.filter(a=>cards.some(c=>(meta(c.id).exercise_refs||[]).includes(a.exercise))).length},
    sources:[...new Set(lessons.flatMap(l=>l.metadata.theory_refs||[]))].map(id=>({title:meta(id)?.title||id,href:recordLink(id)})),
    historyDigest:state.history_digest};
}
