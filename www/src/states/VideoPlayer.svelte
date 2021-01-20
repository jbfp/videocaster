<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import * as server from "../server";
    import MuteButton from "../MuteButton.svelte";
    import PlayButton from "../PlayButton.svelte";
    import SeekBar from "../SeekBar.svelte";

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

    const dispatch = createEventDispatcher();

    export let filePath: string;
    export let subtitlesUrl: string | null;

    $: fileName = filePath.split("__sep").pop();
    $: videoPath = `/video/${encodeURIComponent(filePath)}`;
    $: subtitlesPath = subtitlesUrl
        ? `/subtitles/download/${encodeURIComponent(subtitlesUrl)}`
        : "/subtitles/default";

    let playerState: string | null = null;
    let currentTime: number | null = null;
    let duration: number | null = null;
    let isSeeking = false;
    let canSeek = true;
    let volume = 0.5;
    let isMuted = false;

    $: volumeStr = `${Math.trunc(100 * volume)}%`;

    let player = new RemotePlayer();
    let playerController = new RemotePlayerController(player);

    onMount(async () => {
        //
        // cast setup
        //

        const context = CastContext.getInstance();

        context.setOptions({
            receiverApplicationId: DEFAULT_MEDIA_RECEIVER_APP_ID,
            autoJoinPolicy: AutoJoinPolicy.ORIGIN_SCOPED,
            resumeSavedSession: true,
        });

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

        await loadMedia();
    });

    async function loadMedia() {
        const context = CastContext.getInstance();

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

        const localIp = await server.getLocalIpAsync();

        console.info("local ip", localIp);
        console.info("playing video", filePath, videoPath);
        console.info("subtitles path", subtitlesUrl, subtitlesPath);

        const sub = new Track(1, TrackType.TEXT);
        sub.trackContentId = `http://${localIp}:8080${subtitlesPath}`;
        sub.trackContentType = "text/vtt";
        sub.subtype = TextTrackType.SUBTITLES;
        sub.name = "English Subtitles";
        sub.language = "en-US";
        sub.customData = null;

        const mediaInfo = new MediaInfo(`http://${localIp}:8080${videoPath}`);
        mediaInfo.contentType = "video/mp4";
        mediaInfo.metadata = new MovieMediaMetadata();
        mediaInfo.streamType = StreamType.BUFFERED;
        mediaInfo.textTrackStyle = new TextTrackStyle();
        mediaInfo.duration = null;
        mediaInfo.tracks = [sub];

        const loadRequest = new LoadRequest(mediaInfo);

        try {
            await castSession.loadMedia(loadRequest);
            console.info("media loaded");
        } catch (e) {
            console.error("failed to load media", e);
            return;
        }

        if (subtitlesUrl) {
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

    function setVolume(e: Event) {
        if (!player.canControlVolume) {
            console.warn("cannot control volume");
            return false;
        }

        const previousVolumeLevel = player.volumeLevel;
        const newVolumeLevel = volume;
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

    async function reload() {
        await loadMedia();
    }

    function stop() {
        playerController.stop();
        dispatch("stop");
    }
</script>

<main>
    <h2>Now Playing <code>{fileName}</code></h2>

    <div class="controls">
        <PlayButton {playerState} on:playOrPause={playOrPause} />
        <MuteButton {isMuted} on:muteOrUnmute={muteOrUnmute} />

        <input
            type="range"
            min="0"
            max="1"
            bind:value={volume}
            step="0.01"
            on:change={setVolume}
        />
        <div class="center">{volumeStr}</div>

        <button on:click={reload}>Reload</button>
        <button on:click={stop}> Stop </button>
    </div>

    <div id="seekbar">
        <SeekBar
            {canSeek}
            {currentTime}
            {duration}
            on:seek={seek}
            on:finishSeek={finishSeek}
        />
    </div>
</main>

<style>
    .center {
        display: flex;
        align-items: center;
    }

    .controls {
        display: flex;
        gap: 0.25em;
    }

    #seekbar {
        margin-top: 0.5em;
    }
</style>
