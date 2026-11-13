import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir, realpath, copyFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const MAX_PASSAGE_CHARS = 8000;
const digest = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const validId = s => typeof s === 'string' && /^[a-z0-9][a-z0-9-]*$/.test(s);
const ranges = pages => {
  const result = [];
  for (const page of pages) {
    const last = result.at(-1);
    if (last && page === last[1] + 1) last[1] = page;
    else result.push([page, page]);
  }
  return result.map(([a,b]) => a === b ? `${a}` : `${a}-${b}`);
};

// Keep each original page text intact across one or more paragraph-boundary segments.
export function segmentPages(pages, limit = MAX_PASSAGE_CHARS) {
  const out = [];
  let current = null;
  let heading = '';
  let clear = false;
  const flush = () => { if (current) out.push(current); current = null; };
  for (const page of pages) {
    if (page.state !== 'text') { flush(); continue; }
    const parts = page.text.match(/[^]*?(?:\n[ \t]*\n|$)/g).filter(Boolean);
    for (const part of parts) {
      const first = part.trim().split('\n')[0];
      const recognized = /^(?:#{1,4}\s+\S|(?:chapter|section)\s+\d+\b)/i.test(first) && first.length <= 160;
      if (recognized) { flush(); heading = first; clear = true; }
      if (current && current.length + part.length > limit) flush();
      if (!current) current = { heading, clear, pieces: [], length: 0, oversized: false };
      current.pieces.push({ page: page.page, text: part });
      current.length += part.length;
      if (part.length > limit) current.oversized = true;
    }
  }
  flush();
  return out;
}

export function validateExtraction(x) {
  if (!x || typeof x.extractor !== 'string' || !x.extractor || typeof x.extractor_version !== 'string' || !x.extractor_version || !Array.isArray(x.pages)) throw Error('Invalid extraction envelope');
  if (x.page_count === null) {
    if (x.pages.length || typeof x.document_failure !== 'string' || !x.document_failure) throw Error('Unknown page count requires a document failure and no pages');
    return;
  }
  if (!Number.isSafeInteger(x.page_count) || x.page_count < 1 || x.pages.length !== x.page_count || x.document_failure) throw Error('Incomplete page accounting');
  x.pages.forEach((p, i) => {
    if (p.page !== i + 1 || !['text','no_text','failed'].includes(p.state)) throw Error('Invalid page state or order');
    if (p.state === 'text' && (typeof p.text !== 'string' || !p.text.trim())) throw Error('Text page lacks text');
    if (p.state === 'failed' && (typeof p.reason !== 'string' || !p.reason.trim())) throw Error('Failed page lacks reason');
  });
}
const yaml = (meta, body) => `---\n${Object.entries(meta).map(([k,v]) => `${k}: ${JSON.stringify(v)}`).join('\n')}\n---\n${body}\n`;
const relative = (from, target) => path.posix.relative(path.posix.dirname(from), target);
export function buildChangeset(options, extraction, sourceDigest, createdAt) {
  validateExtraction(extraction);
  const {sourceId, runId, title, edition, language, existingSource} = options;
  if (![sourceId,runId].every(validId) || ![title,edition,language].every(v => typeof v === 'string' && v.trim())) throw Error('Invalid identity or source metadata');
  const source = `sources/${sourceId}.md`, manifest = `manifests/${sourceId}/${runId}.md`;
  const artifact = `references/books/${sourceId}/${sourceDigest.slice(7)}.pdf`;
  const sourceRef = `/${source.slice(0,-3)}`, manifestRef = `/${manifest.slice(0,-3)}`;
  const pages = extraction.pages;
  const segments = segmentPages(pages);
  const byState = state => ranges(pages.filter(p => p.state === state).map(p => p.page));
  const status = !segments.length ? 'failed' : pages.some(p => p.state !== 'text') ? 'partial' : 'complete';
  const ops = [];
  const provenance = from => [{resource: relative(from, artifact), id: sourceId}];
  if (!existingSource) ops.push({op:'create',path:source,content:yaml({type:'Source',schema_version:1,title,location:artifact,format:'pdf',content_digest:sourceDigest,edition,source_language:language,sources:provenance(source)}, 'Retained source PDF. Text extraction does not inspect diagrams.')});
  const refs = [];
  segments.forEach((segment,i) => {
    const first = segment.pieces[0].page, last = segment.pieces.at(-1).page;
    const record = `passages/${sourceId}/${runId}/p${first}-p${last}-part${String(i+1).padStart(4,'0')}.md`;
    refs.push(`/${record.slice(0,-3)}`);
    let body = '';
    let previous = 0;
    for (const piece of segment.pieces) {
      if (piece.page !== previous) { body += `\n<!-- PDF page ${piece.page} -->\n`; previous = piece.page; }
      body += piece.text;
    }
    const meta = {type:'SourcePassage',schema_version:1,title:`${title}: pages ${first}–${last}, part ${i+1}`,version:runId,source_ref:sourceRef,source_digest:sourceDigest,manifest_ref:manifestRef,pdf_page_start:first,pdf_page_end:last,segmentation_status:'review_required',index_status:'extracted',review_status:'unreviewed',sources:provenance(record)};
    if (segment.heading) meta.heading = segment.heading;
    ops.push({op:'create',path:record,content:yaml(meta, body)});
  });
  const meta = {type:'IndexManifest',schema_version:1,title:`${title}: extraction ${runId}`,source_ref:sourceRef,source_digest:sourceDigest,run_id:runId,extractor:extraction.extractor,extractor_version:extraction.extractor_version,page_count:extraction.page_count ?? 0,indexed_ranges:byState('text'),no_text_ranges:byState('no_text'),failed_ranges:byState('failed'),passage_refs:refs,manifest_status:status,created_at:createdAt,sources:provenance(manifest)};
  if (extraction.document_failure) meta.document_failure = extraction.document_failure;
  const failures = pages.filter(p => p.state === 'failed').map(p => `Page ${p.page}: ${p.reason}`);
  ops.push({op:'create',path:manifest,content:yaml(meta, ['Extraction only; passages and diagrams remain unreviewed.',...failures].join('\n\n'))});
  return {changeset:{version:1,operations:ops},artifact,status,passages:refs.length};
}

async function privateDir(dir) {
  const abs = path.resolve(dir);
  await mkdir(abs,{recursive:true,mode:0o700});
  const actual = await realpath(abs);
  const workspace = process.env.LEARNING_WORKSPACE ? await realpath(process.env.LEARNING_WORKSPACE) : process.cwd();
  for (const name of ['courses','learner']) {
    const prohibited = path.join(workspace,name);
    if (actual === prohibited || actual.startsWith(prohibited + path.sep)) throw Error('Staging must be outside canonical learning storage');
  }
  return actual;
}
export async function prepare(options) {
  const dir = await privateDir(options.staging);
  const receiptPath = path.join(dir,'receipt.json');
  try { await readFile(receiptPath); throw Error('Receipt already exists; use verify and retry the existing changeset'); } catch(e) { if (e.code !== 'ENOENT') throw e; }
  let bytes;
  try { bytes = await readFile(options.pdf); }
  catch (error) {
    const receipt = {state:'preflight_failed',pdf:path.resolve(options.pdf),reason:`${error.code || 'READ_ERROR'}: ${error.message}`};
    await writeFile(receiptPath,JSON.stringify(receipt,null,2)+'\n',{flag:'wx',mode:0o600});
    return receipt;
  }
  const sourceDigest = digest(bytes);
  const stagedPdf = path.join(dir,'source.pdf');
  await writeFile(stagedPdf,bytes,{flag:'wx',mode:0o600});
  const script = fileURLToPath(new URL('../read-pdf.sh',import.meta.url));
  const proc = spawnSync('sh',[script,stagedPdf,'--json'],{encoding:'utf8',maxBuffer:512*1024*1024});
  let extraction;
  if (proc.status !== 0) extraction = {extractor:'PDFKit',extractor_version:'read-pdf-json-v1',page_count:null,pages:[],document_failure:`Extractor unavailable or failed: ${(proc.stderr || proc.error?.message || `exit ${proc.status}`).trim()}`};
  else { try { extraction = JSON.parse(proc.stdout); } catch { throw Error('Extractor returned invalid JSON; staged input retained for investigation'); } }
  const built = buildChangeset(options,extraction,sourceDigest,new Date().toISOString());
  const receipt = {state:'pending_publication',pdf:path.resolve(options.pdf),source_digest:sourceDigest,staged_pdf:stagedPdf,source_id:options.sourceId,run_id:options.runId,existing_source:!!options.existingSource,...built};
  await writeFile(receiptPath,JSON.stringify(receipt,null,2)+'\n',{flag:'wx',mode:0o600});
  return receipt;
}
export async function verify(staging, retained) {
  const dir = await privateDir(staging);
  const receipt = JSON.parse(await readFile(path.join(dir,'receipt.json'),'utf8'));
  if (receipt.state !== 'pending_publication') throw Error('No extraction run exists for this preflight failure');
  for (const file of [receipt.pdf,receipt.staged_pdf,...(retained ? [retained] : [])]) {
    if (digest(await readFile(file)) !== receipt.source_digest) throw Error(`Source digest drift: ${file}`);
  }
  return receipt;
}
export async function retain(staging, bundle) {
  const receipt = await verify(staging);
  const root = await realpath(bundle);
  const target = path.join(root,receipt.artifact);
  await mkdir(path.dirname(target),{recursive:true});
  const parent = await realpath(path.dirname(target));
  if (!parent.startsWith(root + path.sep)) throw Error('Artifact path escapes bundle');
  try { await copyFile(receipt.staged_pdf,target,constants.COPYFILE_EXCL); }
  catch (e) { if (e.code !== 'EEXIST') throw e; }
  await verify(staging,target);
  return target;
}
