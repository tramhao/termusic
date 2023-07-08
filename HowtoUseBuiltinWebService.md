## Build with built-in web service enabled
> built-in web service is based on axum crate

```
$ cargo build  --features=webservice --release
```

## Newly added web service cli params
```
$ termusic
...omit unrelated output...
Options:
  -w, --web-service-addr <WEB_SERVICE_ADDR>
          Web service listening addr:port Start the web service if this param is given. For example: 127.0.0.1:3000
  -t, --web-service-token <WEB_SERVICE_TOKEN>
          Mandatory if --web-service-addr is provided Web service will handle client's requests if client provide correct token Token len must be 32
...omit unrelated output...
```

## Sample command to start termusic with built-in web service
```
$ target/release/termusic -w 127.0.0.1:3000 -t ff1432f3c73b4bfd265bc6f7abdaf9ae
```

## Web API list
> If token is not provided in request header or it's not valid, web api will response 401 http code. You can call curl -v to see the verbose information

1. APIs for only performing single action (no need to call {web service addr}/get_status to retrieve the data)
    > Regarding the response JSON payload. The 'result' field will be false and 'message' field will be set to failure information if action gets failed.

    > For example: {"result":false,"message":"Failed to send message to ui main loop","key":null}

    > Don't forget to change authorization token (ff1432f3c73b4bfd265bc6f7abdaf9ae) to your token specified by --web-service-token in sample curl command otherwise you will get 401 error

    #### !! Potential exception
    1. `Not found key in json payload` => json request payload doesn't have `key` field
    2. `Not found this key, did you call /action to get player's info or did you provide correct 'key'?` => this is internal error that indicates webservice thread can't find items in hashmap which is used for storing player status requested by /action with GetXXX operations
    3. `Failed to lock state.locker to send message to ui main loop` => this is internal error that indicates webservice thread can't get locker of sender  in shared state for sending message to ui main thread. See: `src/webservice/mod.rs`

    ### API for general player action (/action)
    ```
    $ curl -X POST http://localhost:3000/action -H "Content-Type:application/json" -d '{"op":"PlayerStart"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","key":null}
    ```
    > You can replace 'PlayerStart' in above command by strings as below to performance different player/playlist operations

    1. PlayerStart - unpause the player
    2. PlayerStop - pause the player
    3. PlayerTogglePause - toggle pause or unpause player
    4. PlayerToggleGapless - toggle gapless option
    5. PlayerNext - play next track
    6. PlayerPrev - play previous track
    7. PlayerVolumeUp - turn up the volume
    8. PlayerVolumeDown - turn down the volume
    9. ChangeLoopModeQueue - change play list loop mode to queue
    10. ChangeLoopModePlaylist - change play list loop mode to playlist
    11. ChangeLoopModeSingle - change play list loop mode to single
    12. EnablePlaylistAddFront - newly added track will be inserted to head of the play list
    13. EnablePlaylistAddTail - newly added track will be inserted to tail of the play list
    14. PlayerSpeedUp - increase the playing speed
    15. PlayerSpeedDown - decrease the playing speed
    16. PlayerEnableGapless - enable player gapless
    17. PlayerDisableGapless - disable player gapless


2. APIs for retrieving player status or data (call /action then call /get_status)
    #### Parameters (json)
    * `op` field in request json payload is `mandatory` for /action web api
    * `op` field in request json payload is ignored for /get_status web api 
    * `key` field in request json payload is ignored for /action web api
    * `key` field in request json payload is `mandatory` for /get_status web api

    #### !! Potential exception
    1. `Failed to send message to ui main loop` => this is internal error that indicates webservice thread can't send message to ui main thread via channel
    2. `Failed to lock state.locker to send message to ui main loop` => this is internal error that indicates webservice thread can't get locker of sender  in shared state for sending message to ui main thread. See: `src/webservice/mod.rs`

    ### API list
    1. GetPlayerStatus
    ```
    $ curl -X POST http://localhost:3000/action -H "Content-Type:application/json" -d '{"op":"GetPlayerStatus"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"      
    {"result":true,"message":"","key":"wxPluYqlrLint0B6hzXY9eOemeWg6D9t"}
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"wxPluYqlrLint0B6hzXY9eOemeWg6D9t"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","is_playing":true}
    ```
    2. GetCurrentTrack
    ```
    $ curl -X POST http://localhost:3000/action -H "Content-Type:application/json" -d '{"op":"GetCurrentTrack"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","key":"AlQBli2GfhDHQk5zs0cneBVcknMreJUm"}
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"AlQBli2GfhDHQk5zs0cneBVcknMreJUm"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","artist":"Unsupported?","album":"Unsupported?","title":"lGZnQyjThxi8oWwp8uC6I6VoragtLCsv","duration":151,"file_path":"/tmp/lGZnQyjThxi8oWwp8uC6I6VoragtLCsv.mp3"}
    ```
    3. GetPlayerGapless
    ```
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"5oqCKIrOCgZiNvEOWMXcW8hk6WVSwMnR"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","key":"5oqCKIrOCgZiNvEOWMXcW8hk6WVSwMnR"}
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"5oqCKIrOCgZiNvEOWMXcW8hk6WVSwMnR"}' -H "authorization: 1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","is_gapless":true}
    ```
    4. GetNextTrack
    ```
    $ curl -X POST http://localhost:3000/action -H "Content-Type:application/json" -d '{"op":"GetNextTrack"}' -H "authorization: ff1432f3c73b4bfd265bc6f7abd af9ae"
    {"result":true,"message":"","key":"zJ2B66B6vyh9xe8UOXh3fXK6YedzABqK"}
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"zJ2B66B6vyh9xe8UOXh3fXK6YedzABqK"}' -H "authorization: ff 1432f3c73b4bfd265bc6f7abdaf9ae" 
    {"result":true,"message":"","artist":"Unsupported?","album":"Unsupported?","title":"lGZnQyjThxi8oWwp8uC6I6VoragtLCsv","duration":151,"file_path":"/tmp/l GZnQyjThxi8oWwp8uC6I6VoragtLCsv.mp3"}

    ```
    5. // GetPrevTrack (not implemented yet)
    6. GetCurrentLoopMode
    ```
    $ curl -X POST http://localhost:3000/action -H "Content-Type:application/json" -d '{"op":"GetCurrentLoopMode"}' -H "authorization: ff1432f3c73b4bfd265bc 6f7abdaf9ae"
    {"result":true,"message":"","key":"PYRnKgWI0IabJNohWHztEAHh99rFa5sS"}
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"PYRnKgWI0IabJNohWHztEAHh99rFa5sS"}' -H "authorization: ff 1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","loop_mode":"Playlist"}
    ```
    7. GetPlaylistAddFront
    ```
    $ curl -X POST http://localhost:3000/action -H "Content-Type:application/json" -d '{"op":"GetPlaylistAddFront"}' -H "authorization: ff1432f3c73b4bfd265b c6f7abdaf9ae"
    {"result":true,"message":"","key":"O6ErUt59jubNCzxc7LLyfAMVlLiJOEJ4"}
    $ curl -X POST http://localhost:3000/get_status -H "Content-Type:application/json" -d '{"key":"O6ErUt59jubNCzxc7LLyfAMVlLiJOEJ4"}' -H "authorization: ff 1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"","add_front":false}
    ```

3. API for adding track (given file path directly) (/add_track)
    #### Parameters (json)
    * `play_now` in json request payload controls if player needs to play newly added track immediately or not
    * `path` in json request payload is the full file path in file system that termusic is running on. For example: `/tmp/Try.mp3`

    #### !! Potential exception
    1. `path doesn't exist!` => full file path specified in `path` field doesn't exist in file system that termusic is running on
    2. `Failed to send message to ui main loop` => this is internal error that indicates webservice thread can't send message to ui main thread via channel
    3. `Failed to lock state.locker to send message to ui main loop` => this is internal error that indicates webservice thread can't get locker of sender  in shared state for sending message to ui main thread. See: `src/webservice/mod.rs`

    ### API
    ```
    # assuming /tmp/Shutterbug.mp3 does exists in same file system where termusic is running on
    $ curl -X POST http://localhost:3000/add_track -H "Content-Type:application/json" -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae" -d '{"path":"/tmp
    /Shutterbug.mp3", "play_now": true}'
    {"result":true,"message":"","key":null}
    ```
4. API for uploading track (/upload_track)
    #### Parameters (query string)
    * `save_to_music_folder` query string param. 
        * Optional. Default is `false`.
        * `true` => save to first folder (suffix with Music) defined in termusic configuration (`Settings.music_dir`) 
            * `file_name` will be normalized and saved (See: https://docs.rs/filenamify/0.1.0/filenamify/)
        * `false` => file name will be replaced with 32 characters and save to OS temp folder. (Linux/MacOS: `/tmp`, Windows for example: `%USERPROFILE%\AppData\Local\Temp\`)
    * `file_name` query string param
        * Mandatory. webapi will verify if file type is supported and valid.
        * `file_name` can't include path. For example `/tmp/try.mp3` will trigger an error.
    * `play_now` query string param
        * Optional. Default is `false`
        * `true` => play immediately once upload is done
        * `false` => just add uploaded track to play list but not play the track
    * The actual media file should exist on your file system which is currently used for calling curl command. 

    #### !! Potential exception
    1. `Failed to send message to ui main loop` => this is internal error that indicates webservice thread can't send message to ui main thread via channel
    2. `Failed to lock state.locker to send message to ui main loop` => this is internal error that indicates webservice thread can't get locker of sender  in shared state for sending message to ui main thread. See: `src/webservice/mod.rs`
    3. `query param 'file_name' can't be empty` => `file_name` url param is empty
    4. `'file_name' should not contain the path. (can't contains '/' or '\')` => `file_name` can't include path. Correct sample: `try.mp3`
    5. `Failed to get extension of {file_name}` => `file_name` doesn't have extension
    6. `Failed to get music folder.` => Can't get music folder. The order to get the folder: 1. Get folder (suffix with 'Music') in settings.music_dir. 2. Get first folder in settings.music_dir. 3. Get folder by calling UserDirs::new().audio_dir() (See: https://docs.rs/directories/latest/directories/struct.UserDirs.html#method.audio_dir)
    7. `<io related error>` => Indicates that error occurs when writing streambody to a file or copying to destination.

    ### API
    ```
    $ curl -X POST http://localhost:3000/upload_track\?save_to_music_folder\=true\&file_name\=Try.mp3\&play_now\=true --data-binary '@/tmp/Try.mp3' -H "authorization: ff1432f3c73b4bfd265bc6f7abdaf9ae"
    {"result":true,"message":"Succesfully uploading track.","file_name":"Try.mp3", "play_now": true}
    ```

## TODOs
1. To support more termusic player actions
2. To support logging
