import path from 'node:path';

// Completeness is checked before interpreting either hits or an empty response.
export function parseOperation(stdout, {query = false} = {}) {
  const rows = stdout.trim().split('\n').filter(Boolean).map(JSON.parse);
  const first = rows[0], last = rows.at(-1);
  if (first?.kind !== 'operation-start' || first.schema_version !== 1 || first.stream_version !== 1 ||
      last?.kind !== 'operation-complete' || last.schema_version !== 1 || last.outcome !== 'ok' || last.exit_code !== 0 ||
      last.completeness?.delivery !== 'complete' ||
      !['complete', 'not-applicable'].includes(last.completeness?.examination) ||
      !['complete', 'not-applicable'].includes(last.completeness?.matching) ||
      (last.completeness?.content && !['complete', 'not-applicable'].includes(last.completeness.content)) ||
      rows.some(r => r.kind === 'warning')) throw new Error('Incomplete or unsuccessful OKF evidence; absence is not established.');
  if (query) {
    const summary = rows.find(r => r.kind === 'query-summary');
    const hits = rows.filter(r => r.id && r.type);
    if (!summary || summary.scan_complete !== true || summary.partial !== false || summary.has_more !== false || summary.returned !== hits.length)
      throw new Error('Incomplete OKF query; narrow scope or finish pagination before using results.');
  }
  return rows;
}

export function referenceId(owner, ref) {
  if (typeof ref !== 'string' || !ref || /[:#?\\]/.test(ref)) throw new Error('Expected a local library record reference.');
  const id = ref.startsWith('/') ? path.posix.normalize(ref) : path.posix.resolve(path.posix.dirname(owner), ref);
  return id.replace(/\.md$/, '');
}

export function chooseRun(source, manifests, selections = []) {
  const valid = manifests.filter(m => referenceId(m.id, m.source_ref) === source.id &&
    m.source_digest === source.content_digest && ['complete', 'partial'].includes(m.manifest_status) && m.passage_refs?.length &&
    m.effective_status !== 'deprecated' && (!m.stale_after || new Date(m.stale_after) > new Date()));
  const selected = selections.filter(s => referenceId(s.id, s.source_ref) === source.id);
  if (selected.length > 1) throw new Error('Conflicting saved run selections.');
  if (selected.length) {
    const run = valid.find(m => m.id === referenceId(selected[0].id, selected[0].manifest_ref));
    if (!run) throw new Error('Saved run is stale, failed or unavailable; explicit re-index/selection is required.');
    return run;
  }
  // More retained indexed pages win; ties prefer complete coverage and then the
  // earliest stable run. A newer partial/failed run cannot silently supersede it.
  const pages = m => m.indexed_ranges.reduce((n, r) => {
    const [a,b = a] = r.split(/[:-]/).map(Number); return n + b-a+1;
  }, 0);
  valid.sort((a,b) => pages(b)-pages(a) || Number(b.manifest_status === 'complete')-Number(a.manifest_status === 'complete') ||
    a.created_at.localeCompare(b.created_at) || a.id.localeCompare(b.id));
  if (!valid.length) throw new Error('No digest-matching usable terminal run; book evidence is unavailable.');
  return valid[0];
}

export function eligiblePassage(passage, source, manifest, reviews = [], now = new Date()) {
  if (passage.type !== 'SourcePassage' || referenceId(passage.id, passage.source_ref) !== source.id ||
      referenceId(passage.id, passage.manifest_ref) !== manifest.id || passage.source_digest !== source.content_digest ||
      !manifest.passage_refs.some(ref => referenceId(manifest.id, ref) === passage.id) ||
      passage.pdf_page_start < 1 || passage.pdf_page_end < passage.pdf_page_start || passage.pdf_page_end > manifest.page_count ||
      passage.effective_status === 'deprecated' || (passage.stale_after && new Date(passage.stale_after) <= now)) return false;
  const relevant = reviews.filter(r => referenceId(r.id, r.passage_ref) === passage.id);
  const superseded = new Set(relevant.filter(r => r.supersedes).map(r => referenceId(r.id, r.supersedes)));
  const terminal = relevant.filter(r => !superseded.has(r.id));
  if (relevant.length && terminal.length !== 1) return false;
  return passage.review_status !== 'blocked' && terminal[0]?.review_status !== 'blocked';
}

export function retrieveBook(cli, sourceId, text) {
  const hits = args => parseOperation(cli(['search', ...args, '--full-scan', '--json']), {query:true}).filter(r => r.id && r.type);
  const open = id => {
    const rows = parseOperation(cli(['show', id, '--json']));
    const record = rows.find(r => r.id && r.type);
    if (!record) throw new Error('Missing opened record: ' + id);
    return record;
  };
  const source = open(sourceId);
  if (source.type !== 'Source' || !source.content_digest || source.access_status === 'inaccessible' || source.effective_status === 'deprecated' ||
      (source.stale_after && new Date(source.stale_after) <= new Date())) throw new Error('Source is unavailable or stale.');
  const selected = chooseRun(source, hits(['--type','IndexManifest']), hits(['--type','SourceSelection']));
  const manifest = open(selected.id);
  const manifestRows = parseOperation(cli(['show',manifest.id,'--body','--json']));
  const manifestBody = manifestRows.find(r => r.kind === 'body' && r.id === manifest.id)?.body;
  if (typeof manifestBody !== 'string') throw new Error('Missing opened manifest coverage details.');
  const reviews = hits(['--type','PassageReview']);
  const candidates = new Map(hits(['--type','SourcePassage','--text',text]).map(r => [r.id,r]));
  const concepts = hits(['--type','Concept','--text',text,'--in','title,description,frontmatter']);
  for (const concept of concepts) for (const ref of concept.passage_refs || []) {
    const id = referenceId(concept.id, ref);
    if (!candidates.has(id)) candidates.set(id, open(id));
  }
  const passages = [], excluded = [];
  for (const candidate of candidates.values()) {
    // Cross-book concept links are navigation only and do not widen evidence scope.
    if (referenceId(candidate.id, candidate.source_ref) !== source.id) continue;
    const passage = open(candidate.id);
    if (!eligiblePassage(passage, source, manifest, reviews)) { excluded.push(passage.id); continue; }
    const bodyRows = parseOperation(cli(['show',passage.id,'--body','--json']));
    const body = bodyRows.find(r => r.kind === 'body' && r.id === passage.id)?.body;
    if (typeof body !== 'string' || !body.trim()) throw new Error('Missing retained passage body.');
    passages.push({...passage, body, response_check_required:true, diagram_evidence:false});
  }
  return {source, manifest, manifest_body:manifestBody, passages, excluded, search_complete:true,
    limitation:'Lexical search and concept aliases do not prove absence. Extracted text is not semantic or diagram review; check each opened passage before citing it.'};
}
