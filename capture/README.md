# What we found in D:\AutoHotKey\UX\AutoHotkeyUX.exe

AutoHotkeyUX.exe is a program for Windows, built for x86_64 (64-bit). Its internal names were removed (stripped), so most functions will have generated names.

Evidence ev_c0590925965d223545483ae9a01f1f793a1cce1e0ca5bfe1329134a486b99bfe.

```json
{
  "architecture": "x86_64",
  "bits": 64,
  "dotnet_arch": null,
  "entry_point": "0x1400bec3c",
  "format": "pe",
  "format_name": "PE (Windows)",
  "kind": "executable",
  "libraries": [
    "ADVAPI32.dll",
    "COMCTL32.dll",
    "GDI32.dll",
    "KERNEL32.dll",
    "OLEAUT32.dll",
    "PSAPI.DLL",
    "SHELL32.dll",
    "SHLWAPI.dll",
    "USER32.dll",
    "UxTheme.dll",
    "VERSION.dll",
    "WININET.dll",
    "WINMM.dll",
    "WSOCK32.dll",
    "dwmapi.dll",
    "ole32.dll"
  ],
  "next_steps": [
    {
      "tool": "core_overview_show",
      "why": "a one-page summary: sections, imports, counts"
    },
    {
      "tool": "native_functions_list",
      "why": "every function we found"
    },
    {
      "tool": "native_strings_find",
      "why": "find text the program shows or uses"
    },
    {
      "tool": "native_function_study",
      "why": "a full dossier on one function (start with main or the entry point)"
    },
    {
      "tool": "core_feature_find",
      "why": "follow a word you see in the app to the code behind it"
    },
    {
      "tool": "native_pe_resources_list",
      "why": "icons, version information and the manifest"
    }
  ],
  "operating_system": "Windows",
  "plain_english": "AutoHotkeyUX.exe is a program for Windows, built for x86_64 (64-bit). Its internal names were removed (stripped), so most functions will have generated names.",
  "recommended_toolsets": [
    "core",
    "native",
    "formats"
  ],
  "stripped": true,
  "symbol_table_stripped": true,
  "toolchain_hints": [
    {
      "category": "compiler",
      "evidence": "Rich header",
      "id": "msvc",
      "label": "Microsoft Visual C++ toolchain",
      "strength": "strong"
    }
  ]
}
```

## Ghidra

Ghidra couldn't analyse D:\AutoHotKey\UX\AutoHotkeyUX.exe (could not start Ghidra: Ghidra's analysis failed; see C:\Users\XENYTE\AppData\Roaming\craft-anything\engines\ghidra-cache\3880f9ec464dfe78-f2dfab6faedc84f0.log), so decompiling uses the built-in decompiler.
