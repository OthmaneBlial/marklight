#!/usr/bin/env python3
"""Measure installed native executables locally. No network or telemetry."""
import argparse,json,os,platform,statistics,subprocess,time
from pathlib import Path
parser=argparse.ArgumentParser()
parser.add_argument('--desktop',type=Path)
parser.add_argument('--cli',type=Path,default=Path('target/release/marklight'))
parser.add_argument('--document',type=Path,default=Path('fixtures/markdown/gfm.md'))
args=parser.parse_args()
root=Path(__file__).resolve().parent.parent
os.chdir(root)
artifact=root/'artifacts';artifact.mkdir(exist_ok=True)
report={'host':platform.platform(),'machine':platform.machine(),'cpu':subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string'],text=True).strip() if platform.system()=='Darwin' else platform.processor()}
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
    for i in range(3):
        output=artifact/f'native-ready-{i}.json';output.unlink(missing_ok=True)
        env=os.environ.copy();env['MARKLIGHT_BENCH_OUTPUT']=str(output)
        start=time.perf_counter()
        process=subprocess.Popen([str(args.desktop.resolve()),str(args.document.resolve())],env=env)
        try:
            deadline=time.monotonic()+30
            while not output.exists():
                if process.poll() is not None:raise RuntimeError('native process exited before ready signal')
                if time.monotonic()>deadline:raise RuntimeError('no native ready signal within 30 seconds; check existing instances')
                time.sleep(.01)
            native=json.loads(output.read_text());native['observed_wall_ms']=(time.perf_counter()-start)*1000;samples.append(native)
            if i==2:
                time.sleep(3)
                for _ in range(5):
                    begin=time.perf_counter();before=cpu_seconds(process.pid);time.sleep(2)
                    elapsed=time.perf_counter()-begin
                    cpu=(cpu_seconds(process.pid)-before)/elapsed*100
                    rss=int(subprocess.check_output(['ps','-p',str(process.pid),'-o','rss='],text=True).strip())
                    idle.append({'native_cpu_percent':cpu,'native_rss_kib':rss,'interval_seconds':elapsed})
        finally:
            process.terminate()
            try:process.wait(timeout=5)
            except subprocess.TimeoutExpired:process.kill();process.wait()
            time.sleep(.2)
    report['desktop_ready']=samples;report['desktop_idle_native_only']=idle
(artifact/'startup.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
