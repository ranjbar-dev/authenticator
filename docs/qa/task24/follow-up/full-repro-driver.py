"""Owned QA driver: preserve only checked Cargo release directories; no deletion."""
import os,sys,json,time,tempfile,shutil,subprocess,struct
from pathlib import Path
root=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(root/'scripts'))
import package_windows as package
target=(root/'target').resolve()
assert target.is_relative_to(root.resolve()) and target != root.resolve()
owned=Path(tempfile.mkdtemp(prefix='task24-full-repro-',dir=target)).resolve()
assert owned.is_relative_to(target) and owned != target
os.environ['TEMP']=os.environ['TMP']=str(owned)
os.environ['CARGO_BUILD_JOBS']='2'
sources=[target/'release',target/package.TARGET/'release']
allowed={'.cargo-lock','.cargo-artifact-lock','.cargo-build-lock','.fingerprint','build','deps','examples','incremental','Authenticator.exe','Authenticator.d','Authenticator.pdb','authenticator.dll','authenticator.dll.lib','authenticator.pdb','libauthenticator.rlib','authenticator.d'}
def inspect(path):
    resolved=path.resolve()
    if resolved != path or not resolved.is_relative_to(target) or resolved==target or not path.is_dir() or path.is_symlink() or path.is_junction():
        raise RuntimeError('Release source identity is not a real bounded artifact directory')
    top=sorted(p.name for p in path.iterdir())
    if set(top)-allowed or '.fingerprint' not in top or 'deps' not in top:
        raise RuntimeError('Unexpected release directory contents: '+str(top))
    size=0;count=0
    for p in path.rglob('*'):
        if p.is_symlink() or p.is_junction(): raise RuntimeError('Link in release artifact tree')
        if p.is_file():
            if p.name in {'vault.bin','session.lock','ownership.txt'} or p.suffix.lower() in {'.vault','.authpbk','.ready'}:
                raise RuntimeError('Possible QA/user data in release artifacts; preserve in place')
            size+=p.stat().st_size;count+=1
    return {'path':str(resolved),'top':top,'files':count,'bytes':size}
def inputs():
    paths=list((root/'src').rglob('*.rs'))+list((root/'assets').rglob('*'))
    paths += [root/p for p in ['build.rs','Cargo.toml','Cargo.lock','.gitattributes','scripts/package_windows.py','docs/package-readme.md','docs/user-guide.md','docs/security-limits.md','LICENSE-MIT','LICENSE-APACHE']]
    return [{'path':p.relative_to(root).as_posix(),'sha256':package.sha(p)} for p in sorted(paths) if p.is_file()]
baseline=inputs()
(owned/'source-inputs.json').write_text(json.dumps(baseline,indent=2),encoding='utf-8')
report={'owned_root':str(owned),'source_commit':package.command('git','rev-parse','HEAD'),'source_dirty':True,'scope':'Same source/toolchain/SDK/cache and project paths; full host and target release outputs rebuilt; Rust standard library is the installed toolchain input.','preserved_directories':[],'runs':[]}
archive=root/'dist/Authenticator-0.1.0-windows-x86_64.zip'
package.validate(archive)
shutil.copyfile(archive,owned/'preceding-package.zip')
shutil.copyfile(archive.with_suffix('.zip.sha256'),owned/'preceding-package.zip.sha256')
current_log=None
def command(*args):
    result=subprocess.run(args,cwd=root,text=True,encoding='utf-8',capture_output=True)
    if args[:2]==('cargo','build'):
        current_log.write_text(result.stdout+'\n'+result.stderr,encoding='utf-8',newline='\n')
    if result.returncode: raise RuntimeError('Command failed '+str(args)+'\n'+result.stderr[-12000:])
    return result.stdout.strip()
package.command=command
print(json.dumps({'owned_root':str(owned)}),flush=True)
for index in (1,2):
    inventory=[inspect(p) for p in sources]
    free=shutil.disk_usage(target).free
    if free < sum(i['bytes'] for i in inventory)*2+2*1024**3:
        raise RuntimeError('Insufficient safe D-drive rebuild headroom')
    saved=owned/f'before-build-{index}'
    saved.mkdir()
    for label,source,item in zip(('host-release','windows-target-release'),sources,inventory):
        # Recheck immediately before each atomic same-volume preservation move.
        if inspect(source)!=item: raise RuntimeError('Release tree changed before preservation')
        destination=(saved/label).resolve()
        if not destination.is_relative_to(owned) or destination.exists() or source.drive.lower()!=destination.drive.lower():
            raise RuntimeError('Invalid preservation destination')
        os.rename(source,destination)
        if source.exists() or not destination.is_dir(): raise RuntimeError('Preservation move failed')
        report['preserved_directories'].append({'round':index,'source':str(source),'destination':str(destination),'inventory':item,'free_before':free})
    current_log=owned/f'compile-{index}.log'
    started=time.time()
    package.build(True,False)
    log=current_log.read_text(encoding='utf-8')
    for name in ('authenticator','egui','eframe','winresource'):
        if 'Compiling '+name+' v' not in log: raise RuntimeError('Missing fresh compilation: '+name)
    if inputs()!=baseline: raise RuntimeError('Measured source inputs changed between rebuilds')
    binary=target/package.TARGET/'release/Authenticator.exe'
    shutil.copyfile(binary,owned/f'build-{index}.exe')
    shutil.copyfile(archive,owned/f'build-{index}.zip')
    raw=binary.read_bytes();pe=struct.unpack_from('<I',raw,0x3c)[0]
    item={'index':index,'seconds':round(time.time()-started,3),'exe_sha256':package.sha(binary),'zip_sha256':package.sha(archive),'exe_bytes':len(raw),'zip_bytes':archive.stat().st_size,'pe_coff_timestamp':struct.unpack_from('<I',raw,pe+8)[0],'compiling_lines':log.count('Compiling '),'full_host_and_target_release_outputs_empty_before_build':True}
    report['runs'].append(item)
    (owned/'result.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(item),flush=True)
report['exe_identical']=report['runs'][0]['exe_sha256']==report['runs'][1]['exe_sha256']
report['zip_identical']=report['runs'][0]['zip_sha256']==report['runs'][1]['zip_sha256']
(owned/'result.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report),flush=True)
if not report['exe_identical'] or not report['zip_identical']: raise RuntimeError('Full rebuild bytes differ; preserved outputs require diagnosis')
