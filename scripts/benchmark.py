#!/usr/bin/env python3
"""Measure installed native executables locally. No network or telemetry."""
import argparse,hashlib,json,os,platform,re,signal,statistics,subprocess,time
from pathlib import Path
parser=argparse.ArgumentParser()
parser.add_argument('--desktop',type=Path)
parser.add_argument('--cli',type=Path,default=Path('target/release/marklight'))
parser.add_argument('--document',type=Path,default=Path('fixtures/markdown/gfm.md'))
parser.add_argument('--large',action='store_true',help='Also load generated 10 KB–10 MB files in the native app')
parser.add_argument('--repeat-large',type=int,default=1,help='Fresh native launches per generated size (default: 1)')
parser.add_argument('--varied-guide',type=Path,help='Also open a real, varied Markdown guide in a fresh native app')
parser.add_argument('--output',type=Path,default=Path('artifacts/startup.json'),help='Write raw results to this JSON path')
args=parser.parse_args()
if args.repeat_large < 1:parser.error('--repeat-large must be at least 1')
root=Path(__file__).resolve().parent.parent
os.chdir(root)
artifact=root/'artifacts';artifact.mkdir(exist_ok=True)
report_path=args.output.resolve();report_path.parent.mkdir(parents=True,exist_ok=True)
report={'host':platform.platform(),'machine':platform.machine(),'cpu':subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string'],text=True).strip() if platform.system()=='Darwin' else platform.processor(),
        'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        'source_tree_dirty':bool(subprocess.check_output(['git','status','--porcelain'],text=True).strip()),
        'cli_sha256':hashlib.sha256(args.cli.read_bytes()).hexdigest()}
if args.desktop:report['desktop_sha256']=hashlib.sha256(args.desktop.read_bytes()).hexdigest()
for name,command in [('cli_help',[str(args.cli.resolve()),'--help']),('cli_read',[str(args.cli.resolve()),str(args.document.resolve()),'--plain','--no-pager'])]:
    samples=[]
    for _ in range(20):
        start=time.perf_counter();subprocess.run(command,stdout=subprocess.DEVNULL,check=True);samples.append((time.perf_counter()-start)*1000)
    report[name]={'first_ms':samples[0],'median_ms':statistics.median(samples),'max_ms':max(samples),'samples_ms':samples}
def cpu_seconds(pid):
    value=subprocess.check_output(['ps','-p',str(pid),'-o','time='],text=True).strip()
    result=0.0
    for part in value.split(':'):result=result*60+float(part)
    return result
if args.desktop:
    samples=[];idle=[]
    documents=[args.document]*3
    if args.varied_guide:documents.append(args.varied_guide)
    if args.large:documents += [artifact/f'large-{size}.md' for size in [10000,100000,1000000,5000000,10000000] for _ in range(args.repeat_large)]
    for i,document in enumerate(documents):
        output=report_path.with_name(f'{report_path.stem}-ready-{i}.json');output.unlink(missing_ok=True)
        env=os.environ.copy();env['MARKLIGHT_BENCH_OUTPUT']=str(output)
        start=time.perf_counter()
        executable=args.desktop.resolve()
        def app_pids():
            found=subprocess.run(['pgrep','-f','^'+re.escape(str(executable))+r'( |$)'],capture_output=True,text=True)
            return {int(pid) for pid in found.stdout.split()}
        before_pids=app_pids()
        if before_pids:raise RuntimeError('Close existing Marklight instances before benchmarking.')
        if platform.system()=='Darwin' and '.app/' in str(executable):
            # LaunchServices foreground opening is the normal desktop UX.
            # WKWebView animation frames may be suspended in a background app.
            bundle=next(parent for parent in executable.parents if parent.suffix=='.app')
            command=['open','-n','-W','-a',str(bundle),'--env',f'MARKLIGHT_BENCH_OUTPUT={output}','--args',str(document.resolve())]
        else:command=[str(executable),str(document.resolve())]
        process=subprocess.Popen(command,env=env)
        native_pid=None
        try:
            deadline=time.monotonic()+90
            while not output.exists():
                if process.poll() is not None:raise RuntimeError('native process exited before ready signal')
                if time.monotonic()>deadline:raise RuntimeError('no native ready signal within 90 seconds; check existing instances')
                time.sleep(.01)
            native=json.loads(output.read_text())
            if native.get('frontend') is None:
                raise RuntimeError(f'native ready signal arrived before {document.name} finished rendering')
            native_pid=native['pid'];native['observed_wall_ms']=(time.perf_counter()-start)*1000;native['document_bytes']=document.stat().st_size;native['document_name']=document.name;samples.append(native)
            print(json.dumps(native),flush=True)
            if i==2:
                time.sleep(3)
                for _ in range(5):
                    begin=time.perf_counter();before=cpu_seconds(native_pid);time.sleep(2)
                    elapsed=time.perf_counter()-begin
                    cpu=(cpu_seconds(native_pid)-before)/elapsed*100
                    rss=int(subprocess.check_output(['ps','-p',str(native_pid),'-o','rss='],text=True).strip())
                    idle.append({'native_cpu_percent':cpu,'native_rss_kib':rss,'interval_seconds':elapsed})
        finally:
            for pid in (app_pids()-before_pids):
                try:os.kill(pid,signal.SIGTERM)
                except ProcessLookupError:pass
            process.terminate()
            try:process.wait(timeout=5)
            except subprocess.TimeoutExpired:process.kill();process.wait()
            time.sleep(.2)
    report['desktop_ready']=samples;report['desktop_idle_native_only']=idle
report_path.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
