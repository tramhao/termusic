This is Lazarus Pascal example that loads the SoundTouch dynamic-load library
and queries the library version as a simple example how to load SoundTouch from
Pascal / Lazarus.

Set the SoundTouch dynamic library file name in the 'InitDLL' procedure of
file 'SoundTouchDLL.pas' depending on if you're building for Windows or Linux.

The example expects the the 'libSoundTouchDll.so' (linux) or 'SoundTouch.dll' (Windows)
library binary files is found within this project directory, either via soft-link
(in Linux) or as a copied file.
