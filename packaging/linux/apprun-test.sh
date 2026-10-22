#!/usr/bin/env bash
# Execute AppRun against an isolated program, including paths/arguments with spaces.
set -euo pipefail
python3 - "$(dirname "${BASH_SOURCE[0]}")/AppRun" <<'PY'
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

with tempfile.TemporaryDirectory(prefix='vectorcraft AppRun ') as directory:
    root = Path(directory)
    (root / 'usr/bin').mkdir(parents=True)
    (root / 'usr/lib').mkdir()
    shutil.copyfile(sys.argv[1], root / 'AppRun')
    (root / 'AppRun').chmod(0o755)
    binary = root / 'usr/bin/vectorcraft'
    binary.write_text('#!/usr/bin/env python3\nimport json,os,sys\nprint(json.dumps([sys.argv[1:],os.environ["LD_LIBRARY_PATH"]]))\n')
    binary.chmod(0o755)
    for previous in (None, '', '/existing/library path'):
        env = dict(os.environ)
        env.pop('LD_LIBRARY_PATH', None)
        if previous is not None:
            env['LD_LIBRARY_PATH'] = previous
        args = ['--example', 'a file with spaces.svg', 'Unicode-图.svg', '']
        result = subprocess.run([str(root / 'AppRun'), *args], env=env, text=True,
                                capture_output=True, check=True, timeout=10)
        actual_args, path = json.loads(result.stdout)
        assert actual_args == args
        assert path == str(root / 'usr/lib') + (':' + previous if previous else '')
print('AppRun preserves arguments and selects only its private library directory first')
PY
