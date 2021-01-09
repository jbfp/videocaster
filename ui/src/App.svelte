<script lang="ts">
    import FileList from "./FileList.svelte";
    import MuteButton from "./MuteButton.svelte";
    import PlayButton from "./PlayButton.svelte";
    import SeekBar from "./SeekBar.svelte";
    import VolumeBar from "./VolumeBar.svelte";

    const {
        CastContext,
        RemotePlayer,
        RemotePlayerController,
        RemotePlayerEventType,
        // @ts-ignore
    } = cast.framework;

    // @ts-ignore
    const { AutoJoinPolicy } = chrome.cast;

    const {
        EditTracksInfoRequest,
        LoadRequest,
        MediaInfo,
        MovieMediaMetadata,
        StreamType,
        TextTrackStyle,
        TextTrackType,
        Track,
        TrackType,
        DEFAULT_MEDIA_RECEIVER_APP_ID,
        // @ts-ignore
    } = chrome.cast.media;

    let localIp: string | null = null;
    let currentDir: string | null = null;
    let selectedFile: string | null = null;
    let entries = [];

    let playerState: string | null = null;
    let currentTime: number | null = null;
    let duration: number | null = null;
    let isSeeking = false;
    let canSeek = true;
    let volume = 0.5;
    let isMuted = false;

    //
    // client->server functions
    //

    async function getLocalIp() {
        let res: Response;

        try {
            res = await fetch("/ip");
        } catch (e) {
            console.error(e);
            return;
        }

        try {
            localIp = await res.json();
        } catch (e) {
            console.error(e);
            return;
        }

        console.info("local ip", localIp);
    }

    async function loadDir() {
        const path = currentDir
            ? `?path=${encodeURIComponent(currentDir)}`
            : "";

        const url = `/fs${path}`;

        let res: Response;

        try {
            res = await fetch(url);
        } catch (e) {
            console.error(e);
            return;
        }

        let result: {
            items: { isDir: boolean; name: string }[];
            realPath: string;
        };

        try {
            result = await res.json();
        } catch (e) {
            console.error(e);
            return;
        }

        currentDir = result.realPath;
        entries = result.items.map(({ isDir, name }) => ({
            name,
            type: isDir ? "dir" : "file",
            url: `#${name}`,

            onClick() {
                const path = `${currentDir}/${name}`;

                if (isDir) {
                    currentDir = path;
                    loadDir();
                } else {
                    selectedFile = path;
                    loadMedia();
                }
            },
        }));
    }

    getLocalIp();
    loadDir();

    //
    // cast setup
    //

    const context = CastContext.getInstance();

    context.setOptions({
        receiverApplicationId: DEFAULT_MEDIA_RECEIVER_APP_ID,
        autoJoinPolicy: AutoJoinPolicy.ORIGIN_SCOPED,
        resumeSavedSession: true,
    });

    const player = new RemotePlayer();
    const playerController = new RemotePlayerController(player);

    playerController.addEventListener(
        RemotePlayerEventType.PLAYER_STATE_CHANGED,
        (e) => (playerState = e.value)
    );

    playerController.addEventListener(
        RemotePlayerEventType.CAN_SEEK_CHANGED,
        (e) => (canSeek = e.value)
    );

    playerController.addEventListener(
        RemotePlayerEventType.CURRENT_TIME_CHANGED,
        (e) => {
            console.debug(`current time changed to ${e.value}`);

            if (!isSeeking) {
                currentTime = e.value;
            }
        }
    );

    playerController.addEventListener(
        RemotePlayerEventType.DURATION_CHANGED,
        (e) => {
            console.debug(`duration changed to ${e.value}`);
            duration = e.value;
        }
    );

    playerController.addEventListener(
        RemotePlayerEventType.VOLUME_LEVEL_CHANGED,
        (e) => {
            console.debug(`volume level changed to ${e.value}`);
            volume = e.value;
            isMuted = false;
        }
    );

    playerController.addEventListener(
        RemotePlayerEventType.IS_MUTED_CHANGED,
        (e) => (isMuted = e.value)
    );

    playerController.addEventListener(
        RemotePlayerEventType.TITLE_CHANGED,
        (e) => (document.title = `Videocaster - ${e.value}`)
    );

    //
    // callbacks
    //

    async function loadMedia() {
        let castSession = context.getCurrentSession();

        if (!castSession) {
            console.info("getting cast session...");

            try {
                await context.requestSession();
            } catch (e) {
                return;
            }

            castSession = context.getCurrentSession();
        }

        console.info("playing video", selectedFile);

        const sub = new Track(1, TrackType.TEXT);
        sub.trackContentId = `http://${localIp}:8080/subtitles/${encodeURIComponent(
            selectedFile
        )}`;
        sub.trackContentType = "text/vtt";
        sub.subtype = TextTrackType.SUBTITLES;
        sub.name = "English Subtitles";
        sub.language = "en-US";
        sub.customData = null;

        const tracks = [sub];
        const mediaInfo = new MediaInfo(
            `http://${localIp}:8080/video/${encodeURIComponent(selectedFile)}`
        );
        mediaInfo.contentType = "video/mp4";
        mediaInfo.metadata = new MovieMediaMetadata();
        mediaInfo.streamType = StreamType.BUFFERED;
        mediaInfo.textTrackStyle = new TextTrackStyle();
        mediaInfo.duration = null;
        mediaInfo.tracks = tracks;

        const loadRequest = new LoadRequest(mediaInfo);

        try {
            await castSession.loadMedia(loadRequest);
            console.info("media loaded");
        } catch (e) {
            console.error("failed to load media", e);
            return;
        }

        let media;

        try {
            media = await castSession.getMediaSession();
            console.debug("retrieved media session");
        } catch (e) {
            console.error("failed to load media session", e);
            return;
        }

        const tracksInfoRequest = new EditTracksInfoRequest([1]);

        try {
            await media.editTracksInfo(tracksInfoRequest);
            console.info("subtitles loaded");
        } catch (e) {
            console.warn("failed to set subtitle track", e);
        }

        canSeek = true;
    }

    function seek(e: CustomEvent<number>) {
        isSeeking = true;
        currentTime = e.detail;
    }

    function finishSeek(e: CustomEvent<number>) {
        isSeeking = false;

        if (!player.canSeek) {
            console.warn("cannot seek");
            return;
        }

        const previousTime = player.currentTime;
        const newTime = e.detail;
        player.currentTime = newTime;

        try {
            playerController.seek();
            console.log(`seek from ${previousTime} to ${newTime}`);
        } catch (e) {
            console.error(e);
        }
    }

    function setVolume(e: CustomEvent<number>) {
        if (!player.canControlVolume) {
            console.warn("cannot control volume");
            return false;
        }

        const previousVolumeLevel = player.volumeLevel;
        const newVolumeLevel = e.detail;
        player.volumeLevel = newVolumeLevel;

        try {
            playerController.setVolumeLevel();
            console.log(
                `volume changed from ${previousVolumeLevel} to ${newVolumeLevel}`
            );
        } catch (e) {
            console.error(e);
        }
    }

    function muteOrUnmute() {
        playerController.muteOrUnmute();
    }

    function playOrPause() {
        playerController.playOrPause();
    }
</script>

<style>
    google-cast-launcher {
        display: inline-block;
        width: 24px;
        height: 24px;
    }

    #file-list {
        flex: 1;
        overflow-y: auto;
    }

    #player {
        height: 7vh;
    }

    #sub-notice {
        color: #666666;
        display: inline-block;
        font-size: small;
        margin-top: 2em;
    }
</style>

<h1>
    Videocaster
    <google-cast-launcher id="cast-btn" />
</h1>

<div>{currentDir}</div>
<div id="file-list">
    <FileList {entries} />
</div>

<div hidden={!selectedFile}>
    {selectedFile}
    <button id="start-btn" on:click={loadMedia}>Reload</button>
</div>

<div id="player">
    <div>
        <PlayButton {playerState} on:playOrPause={playOrPause} />
        <MuteButton {isMuted} on:muteOrUnmute={muteOrUnmute} />
        <VolumeBar {volume} on:setVolume={setVolume} />
    </div>

    <div>
        <SeekBar
            {canSeek}
            {currentTime}
            {duration}
            on:seek={seek}
            on:finishSeek={finishSeek} />
    </div>
</div>

<footer>
    <em id="sub-notice">
        Subtitles provided by
        <a href="https://www.opensubtitles.org" target="_blank">
            OpenSubtitles.org
        </a>
    </em>
</footer>
