import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {analyzeObservation} from './observation.mjs';
const capture=()=>JSON.parse(readFileSync(new URL('./fixtures/observed-motion.redacted.json',import.meta.url),'utf8'));
test('recorded movement separates the SDK-matched local truck and the moving peer',()=>{
 const result=analyzeObservation(capture());
 assert.equal(result.metrics.frames,117);assert.equal(result.metrics.sourceRemoteFrames,98);
 assert.equal(result.metrics.missingSourceFrames,19);assert.ok(result.metrics.localMaxDisplacement<0.001);
 assert.ok(Math.abs(result.metrics.remoteMaxDisplacement-39.953)<0.01);
 assert.equal(result.offlineReplay,true);assert.equal(result.completeRosterVerified,false);
 assert.ok(result.frames.every(f=>f.remoteCandidates.every(v=>v.id!=='local')));
});
test('missing vehicle rows and missing SDK ownership clear candidates immediately',()=>{
 const result=analyzeObservation(capture());
 assert.deepEqual(result.frames[55].remoteCandidates,[]);
 assert.deepEqual(result.frames[68].remoteCandidates,[]);
 const missingOwn=capture().samples.findIndex(s=>s.own===null);
 assert.ok(missingOwn>=0);assert.deepEqual(result.frames[missingOwn].remoteCandidates,[]);
 assert.equal(result.frames[missingOwn].reason,'own-telemetry-unavailable');
 assert.equal(result.metrics.longestMissingRunSamples,12);
 assert.ok(result.metrics.longestSourceAbsenceMs>6000);
});
test('overlapping vehicles do not guess which one belongs to the local player',()=>{
 const c=capture(),index=c.samples.findIndex(s=>s.own!==null);c.samples[index].vehicles[1].x=c.samples[index].own.x;c.samples[index].vehicles[1].z=c.samples[index].own.z;
 const r=analyzeObservation(c);assert.equal(r.frames[index].reason,'ambiguous-own-vehicle');assert.deepEqual(r.frames[index].remoteCandidates,[]);
});
test('invalid positions and backwards timestamps reject the whole replay',()=>{
 const c=capture();c.samples[1].vehicles[0].x=NaN;assert.throws(()=>analyzeObservation(c),/invalid-observation/);
 const time=capture();time.samples[1].tMs=-1;assert.throws(()=>analyzeObservation(time),/invalid-observation/);
});
