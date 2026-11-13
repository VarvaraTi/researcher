import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, writeFile, readFile, mkdir, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import * as fs from 'node:fs';
import {buildChangeset, segmentPages, validateExtraction, prepare, verify, retain} from '../scripts/lib/book-ingestion.mjs';

const opt = {sourceId:'book',runId:'run-one',title:'Synthetic',edition:'First',language:'en'};
const sample = {extractor:'synthetic',extractor_version:'1',page_count:4,pages:[{page:1,state:'text',text:'# Heading\n\nFirst paragraph.\n\nSecond paragraph.\n\n'},{page:2,state:'text',text:'Continuation.\n\n'},{page:3,state:'no_text'},{page:4,state:'failed',reason:'damaged stream'}]};

test('every known page is accounted for and page text survives segmentation', () => {
  const segments = segmentPages(sample.pages,25);
  assert(segments.length > 2);
  for (const p of sample.pages.filter(p=>p.state==='text')) assert.equal(segments.flatMap(s=>s.pieces).filter(piece=>piece.page===p.page).map(piece=>piece.text).join(''),p.text);
  const built = buildChangeset(opt,sample,'sha256:abcd','2026-01-01T00:00:00Z');
  assert.equal(built.status,'partial');
  const manifest = built.changeset.operations.at(-1).content;
  assert.match(manifest,/indexed_ranges: \["1-2"\]/);
  assert.match(manifest,/no_text_ranges: \["3"\]/);
  assert.match(manifest,/failed_ranges: \["4"\]/);
  assert.match(manifest,/damaged stream/);
  assert.match(built.changeset.operations[1].content,/<!-- PDF page 1 -->/);
  assert.throws(()=>validateExtraction({...sample,pages:sample.pages.slice(0,3)}),/accounting/);
});
test('cross-page passages and oversized paragraphs retain exact page text and flag uncertainty', () => {
  const pages=[{page:1,state:'text',text:'First.\n\n'},{page:2,state:'text',text:'Second.\n\n'}];
  const segment=segmentPages(pages)[0];
  assert.deepEqual(segment.pieces.map(x=>x.page),[1,2]);
  const oversized=segmentPages([{page:1,state:'text',text:'a'.repeat(8100)}])[0];
  assert.equal(oversized.pieces[0].text.length,8100);
  assert(oversized.oversized);
  assert.equal(segment.clear,false);
});
test('unknown count has an explicit failed manifest and no invented ranges', () => {
  const result=buildChangeset(opt,{extractor:'PDFKit',extractor_version:'1',page_count:null,pages:[],document_failure:'locked'},'sha256:abc','2026-01-01T00:00:00Z');
  assert.equal(result.status,'failed');
  assert.match(result.changeset.operations.at(-1).content,/page_count: 0/);
  assert.match(result.changeset.operations.at(-1).content,/document_failure: "locked"/);
  assert.throws(()=>validateExtraction({extractor:'x',extractor_version:'1',page_count:null,pages:[]}),/document failure/);
});
test('preflight failure does not invent a digest or run; retry checks source drift', async t => {
  const root=await mkdtemp(path.join(tmpdir(),'book-test-')); t.after(()=>rm(root,{recursive:true,force:true}));
  const failed=await prepare({...opt,pdf:path.join(root,'missing'),staging:path.join(root,'failed')});
  assert.equal(failed.state,'preflight_failed'); assert.equal(failed.source_digest,undefined); assert.equal(failed.run_id,undefined);
  const pdf=path.join(root,'input.pdf'); await writeFile(pdf,'synthetic bytes');
  const staging=path.join(root,'stage');
  const receipt=await prepare({...opt,pdf,staging});
  assert.equal(receipt.state,'pending_publication');
  await assert.rejects(prepare({...opt,pdf,staging}),/already exists/);
  const bundle=path.join(root,'library'); await mkdir(bundle);
  const artifact=await retain(staging,bundle); await retain(staging,bundle);
  assert.equal((await verify(staging,artifact)).run_id,'run-one');
  await writeFile(pdf,'changed');
  await assert.rejects(verify(staging,artifact),/digest drift/);
  const saved=JSON.parse(await readFile(path.join(staging,'receipt.json')));
  assert.deepEqual(saved.changeset,receipt.changeset);
});
test('PDFKit extraction integration when available', t => {
  if (process.platform !== 'darwin') return t.skip('PDFKit requires macOS');
  const swift=spawnSync('swift',['--version'],{encoding:'utf8'});
  if (swift.status !== 0) return t.skip('Swift unavailable');
  const dir=fs.mkdtempSync(path.join(tmpdir(),'pdfkit-test-'));
  const pdf=path.join(dir,'fixture.pdf');
  const objects = ['<< /Type /Catalog /Pages 2 0 R >>','<< /Type /Pages /Kids [3 0 R] /Count 1 >>','<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>','<< /Length 39 >>\nstream\nBT /F1 12 Tf 10 50 Td (Hello) Tj ET\nendstream','<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>'];
  let data='%PDF-1.4\n'; const offsets=[0];
  objects.forEach((object,i)=>{offsets.push(Buffer.byteLength(data)); data+=`${i+1} 0 obj\n${object}\nendobj\n`;});
  const xref=Buffer.byteLength(data); data+=`xref\n0 ${offsets.length}\n0000000000 65535 f \n`;
  for (const offset of offsets.slice(1)) data+=`${String(offset).padStart(10,'0')} 00000 n \n`;
  data+=`trailer\n<< /Size ${offsets.length} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  fs.writeFileSync(pdf,data);
  const extraction=spawnSync('sh',['scripts/read-pdf.sh',pdf,'--json'],{encoding:'utf8',env:{...process.env,LEARNING_WORKSPACE:dir}});
  assert.equal(extraction.status,0,extraction.stderr);
  const parsed=JSON.parse(extraction.stdout); validateExtraction(parsed);
  assert.equal(parsed.page_count,1); assert.equal(parsed.pages[0].state,'text'); assert.match(parsed.pages[0].text,/Hello/);
  fs.rmSync(dir,{recursive:true,force:true});
});
