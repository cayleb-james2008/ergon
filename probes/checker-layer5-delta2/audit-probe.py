import sys, json, pathlib
sys.path.insert(0, '/tmp/checker-layer5fix/repo/tools')
sys.path.insert(0, '/tmp/checker-layer5fix/repo/tests/tools')
import test_vinegar_run as T

def run_case(name, script=None):
    t = T.FixtureTests('test_atlas_duplicate_capture_retries_once_and_keeps_the_attempts')
    t.setUp()
    try:
        fake, ctx = t.atlas()
        if script is not None:
            fake.capture_script = script
        code, result = t.run_fixture(ctx, 'atlas:scout')
        p = pathlib.Path(result['runDir']) / 'capture-attempts.json'
        audit = json.loads(p.read_text()) if p.is_file() else None
        res = result.get('captureAttempts', [])
        return {
            'case': name, 'exit': code,
            'error': (result.get('error') or '')[:120],
            'result_entries': len(res), 'result_last_phase': res[-1]['phase'] if res else None,
            'audit_present': audit is not None, 'audit_entries': len(audit) if audit is not None else None,
            'audit_last_phase': audit[-1]['phase'] if audit else None,
            'equal': audit == res,
            'length_equal': (len(audit) == len(res)) if audit is not None else None,
            'content_equal': (audit == res) if audit is not None else None,
        }
    finally:
        t.doCleanups()

def early_retry_later_fail(f):
    if f.captures == 2: return 1          # view-002 first attempt duplicates view-001 -> retry
    if f.captures == 3: return 2          # view-002 retry fresh
    if f.captures in (21, 22, 23): return 20  # view-020 overhead frozen on view-019's bytes -> 3 duplicates
    return f.captures

def success_with_retry(f):
    if f.captures == 2: return 1
    if f.captures == 3: return 2
    return f.captures

out = [run_case('early_retry_then_later_failure', early_retry_later_fail),
       run_case('success_with_retry', success_with_retry),
       run_case('no_retry_success', None)]
print(json.dumps(out, indent=1))
