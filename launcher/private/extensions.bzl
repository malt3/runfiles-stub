"""Module extension for downloading non-module dependencies."""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

_download_attrs = {
    "finalize-stub-aarch64-linux": {
        "name": "finalize_stub_aarch64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/finalize-stub-aarch64-linux",
        "sha256": "5dbe0832107d23c1c931910518a3bd6d6023b6d6ac8c7ce097b29d9b5e11dbb2",
    },
    "finalize-stub-aarch64-macos": {
        "name": "finalize_stub_aarch64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/finalize-stub-aarch64-macos",
        "sha256": "092a4337ad3ad7b2cb5ee82ceaa890757115ad722f833d813d6ec9e7290f8213",
    },
    "finalize-stub-x86_64-linux": {
        "name": "finalize_stub_x86_64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/finalize-stub-x86_64-linux",
        "sha256": "69faa01b2ebc9bca3f4e698bd96054c6b7fa01d09637de7adeb8e39ac34d62ea",
    },
    "finalize-stub-x86_64-macos": {
        "name": "finalize_stub_x86_64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/finalize-stub-x86_64-macos",
        "sha256": "c880c3b1c745d3fc2d8dfa243ab2b2f9349a5d78390ea3d85b29fc0e688971d0",
    },
    "finalize-stub-x86_64-windows.exe": {
        "name": "finalize_stub_x86_64_windows",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/finalize-stub-x86_64-windows.exe",
        "sha256": "f13233dbe907eb113ba820a6e2366efbfd0331112df2dab3c1ef90ba723b5f01",
    },
    "runfiles-stub-aarch64-linux": {
        "name": "runfiles_stub_aarch64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/runfiles-stub-aarch64-linux",
        "sha256": "69ce4519aefd7d6f94dc30dfde7d59c217d884bfba3918a03d17398201048499",
    },
    "runfiles-stub-aarch64-macos": {
        "name": "runfiles_stub_aarch64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/runfiles-stub-aarch64-macos",
        "sha256": "8bbafb6dd7af975a01b060cf482e75ccea3130079baeb166997a2987c42b7a44",
    },
    "runfiles-stub-x86_64-linux": {
        "name": "runfiles_stub_x86_64_linux",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/runfiles-stub-x86_64-linux",
        "sha256": "4ff307a291efc600a6cafb4e2a75b64721f5523c76f0dd489979e4d081000e17",
    },
    "runfiles-stub-x86_64-macos": {
        "name": "runfiles_stub_x86_64_macos",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/runfiles-stub-x86_64-macos",
        "sha256": "0ed9a223a98a35c497d6426e98142291ad632582e2e6a0d85cf37a357db052e4",
    },
    "runfiles-stub-x86_64-windows.exe": {
        "name": "runfiles_stub_x86_64_windows",
        "url": "https://github.com/malt3/hermetic-launcher/releases/download/binaries-20251217/runfiles-stub-x86_64-windows.exe",
        "sha256": "83da6aabdbd2ac387bb0f0a6c6c3b61dd4b08a34688442958dae491394414481",
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
