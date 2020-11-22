#!/usr/bin/env python3

import json
import sys
import os

scripts_dir = os.path.dirname(os.path.abspath(__file__))
project_dir = os.path.dirname(scripts_dir)

json.dump(
    {
        "rdocCaptureSettings": 1,
        "settings": {
            "autoStart": False,
            "commandLine": "",
            "environment": [],
            "executable": os.path.join(
                project_dir, "target", "release", "openkrosskod"
            ),
            "inject": False,
            "numQueuedFrames": 0,
            "options": {
                "allowFullscreen": True,
                "allowVSync": True,
                "apiValidation": False,
                "captureAllCmdLists": False,
                "captureCallstacks": False,
                "captureCallstacksOnlyDraws": False,
                "debugOutputMute": True,
                "delayForDebugger": 0,
                "hookIntoChildren": False,
                "refAllResources": False,
                "verifyBufferAccess": False,
            },
            "queuedFrameCap": 0,
            "workingDir": project_dir,
        },
    },
    sys.stdout,
    indent=4,
)
sys.stdout.write("\n")
