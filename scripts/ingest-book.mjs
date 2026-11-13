#!/usr/bin/env node
import {prepare, verify, retain} from './lib/book-ingestion.mjs';

const usage = 'Usage: ingest-book.mjs prepare --pdf FILE --staging DIR --source-id ID --run-id ID --title TITLE --edition EDITION --language LANG [--existing-source] | verify --staging DIR [--retained FILE] | retain --staging DIR --bundle DIR';
try {
  const [command,...args] = process.argv.slice(2);
  const values = {};
  const allowed = command === 'prepare' ? ['pdf','staging','source-id','run-id','title','edition','language','existing-source'] : command === 'verify' ? ['staging','retained'] : command === 'retain' ? ['staging','bundle'] : [];
  for (let i=0;i<args.length;i++) {
    const key = args[i].startsWith('--') ? args[i].slice(2) : '';
    if (!allowed.includes(key) || key in values) throw Error(usage);
    if (key === 'existing-source') values[key] = true;
    else { if (!args[i+1] || args[i+1].startsWith('--')) throw Error(usage); values[key] = args[++i]; }
  }
  const required = command === 'prepare' ? ['pdf','staging','source-id','run-id','title','edition','language'] : command === 'verify' ? ['staging'] : command === 'retain' ? ['staging','bundle'] : ['invalid'];
  if (required.some(k => !values[k])) throw Error(usage);
  let result;
  if (command === 'prepare') result = await prepare({pdf:values.pdf,staging:values.staging,sourceId:values['source-id'],runId:values['run-id'],title:values.title,edition:values.edition,language:values.language,existingSource:values['existing-source']});
  if (command === 'verify') result = await verify(values.staging,values.retained);
  if (command === 'retain') result = {retained:await retain(values.staging,values.bundle)};
  process.stdout.write(JSON.stringify(result,null,2)+'\n');
  if (result.state === 'preflight_failed') process.exitCode = 1;
} catch (error) { console.error(`ingest-book: ${error.message}`); process.exitCode = 1; }
