load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

_download_attrs = {
    "finalize-stub-aarch64-linux": {
        "name": "finalize_stub_aarch64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/finalize-stub-aarch64-linux",
        "sha256": "f359e88589167d1c4ff5b06fe60e18bcc1aa16a21d49f81056b46a2d059c9a64",
    },
    "finalize-stub-aarch64-macos": {
        "name": "finalize_stub_aarch64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/finalize-stub-aarch64-macos",
        "sha256": "af3533ddc9c5ab3460d7b0521d409cbbf8cc8c519bb6ff3d9d3207af7262700f",
    },
    "finalize-stub-x86_64-linux": {
        "name": "finalize_stub_x86_64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/finalize-stub-x86_64-linux",
        "sha256": "dce220b1ba5a2b44e48070a00e2a260923078bfda1b66f6ee584573ba90562cc",
    },
    "finalize-stub-x86_64-macos": {
        "name": "finalize_stub_x86_64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/finalize-stub-x86_64-macos",
        "sha256": "7763aa4f3b8da89428f83d9aed7b94c52dd4288890e7dabc59c0535747fa3ba4",
    },
    "finalize-stub-x86_64-windows.exe": {
        "name": "finalize_stub_x86_64_windows",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/finalize-stub-x86_64-windows.exe",
        "sha256": "eb5adfd38d4861d72416d7ca57fc75771bda08de860a36dbf912b45c0f5737c6",
    },
    "runfiles-stub-aarch64-linux": {
        "name": "runfiles_stub_aarch64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/runfiles-stub-aarch64-linux",
        "sha256": "69ce4519aefd7d6f94dc30dfde7d59c217d884bfba3918a03d17398201048499",
    },
    "runfiles-stub-aarch64-macos": {
        "name": "runfiles_stub_aarch64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/runfiles-stub-aarch64-macos",
        "sha256": "8bbafb6dd7af975a01b060cf482e75ccea3130079baeb166997a2987c42b7a44",
    },
    "runfiles-stub-x86_64-linux": {
        "name": "runfiles_stub_x86_64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/runfiles-stub-x86_64-linux",
        "sha256": "4ff307a291efc600a6cafb4e2a75b64721f5523c76f0dd489979e4d081000e17",
    },
    "runfiles-stub-x86_64-macos": {
        "name": "runfiles_stub_x86_64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/runfiles-stub-x86_64-macos",
        "sha256": "0ed9a223a98a35c497d6426e98142291ad632582e2e6a0d85cf37a357db052e4",
    },
    "runfiles-stub-x86_64-windows.exe": {
        "name": "runfiles_stub_x86_64_windows",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251216/runfiles-stub-x86_64-windows.exe",
        "sha256": "c7c1662aefcb5a0b42bda894da87ab20c9822fa821ebf670a51bd88eefd058c2",
    },
}

def _non_module_dependencies_impl(ctx):
    for filename, attrs in _download_attrs.items():
        http_file(
            name = attrs["name"],
            url = attrs["url"],
            sha256 = attrs["sha256"],
            downloaded_file_path = filename,
            executable = True,
        )
    return ctx.extension_metadata(
        root_module_direct_deps = "all",
        root_module_direct_dev_deps = [],
        reproducible = True,
    )


non_module_dependencies = module_extension(
    implementation = _non_module_dependencies_impl,
)
