# Clean committed Windows package — 109171a

Reviewed source commit 109171ac72e458ac3c34738e828e83b29983ec5a was pushed normally to origin/main. The working tree was clean before and after this package build. Only the authenticator package release outputs were cleaned after resolving and checking the workspace/target/release bounds; the entire target and all QA fixture roots were preserved. The pre-clean EXE was copied into the owned ignored result root before cleaning.

The commands ran with fresh process-local D: TEMP/TMP and CARGO_BUILD_JOBS=2, through the recorded RTK wrapper:

```text
rtk proxy cargo clean --manifest-path D:/Projects/github/authenticator/Cargo.toml -p authenticator --release --target x86_64-pc-windows-msvc --target-dir D:/Projects/github/authenticator/target
rtk proxy python scripts/package_windows.py --replace
```

The compile/link/package phase took 104.26 seconds. Packaged build-info identifies source_commit 109171ac72e458ac3c34738e828e83b29983ec5a and source_dirty=false. Source epoch 1794616978 is the commit timestamp, not the actual build time.

The current ZIP is dist/Authenticator-0.1.0-windows-x86_64.zip, 6,026,391 bytes, SHA256 ee7af10e69420d99da4eb8ceccf8dc3f4cfa7a9a09d0adbca6221bbcd94afe8b. Its EXE is 12,179,456 bytes, SHA256 9656ed317d6b2e36a96cf17ac5c0d2729680cb6abb850941c295f7418272978e, exactly matching the independently reviewed full-rebuild pair and the actual production scan/copy/independent-key recovery/restart smoke. Regenerating clean source metadata changed ZIP bytes; runtime evidence is reused only because EXE bytes match exactly.

[Result](clean-build-109171a.json), [actual build log](clean-build-109171a.log), and [prior full rebuild/runtime review](../task24/follow-up/independent-review.md). Original wrapper and pre-clean EXE remain ignored at target/task24-parent-clean-109171a-kph96tj0. Current hosted [run 43536070275](https://github.com/ranjbar-dev/authenticator/actions/runs/43536070275) subsequently completed successfully, including full tests/build/package/upload; [hosted record](hosted-109171a.md) keeps its artifact identity separate. Five bounded native state cases and external offline/OS/speech gates remain separate.
