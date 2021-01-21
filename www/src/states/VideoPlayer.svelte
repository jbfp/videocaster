<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import * as server from "../server";
    import VideoPlayerView from "./VideoPlayerView.svelte";

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

    let state = {
        playerState: null,
        currentTime: null,
        duration: null,
        canSeek: true,
        volume: 0.5,
        isMuted: false,
    };

    let ready = false;
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
            (e) => (state = { ...state, playerState: e.value })
        );

        playerController.addEventListener(
            RemotePlayerEventType.CAN_SEEK_CHANGED,
            (e) => (state = { ...state, canSeek: e.value })
        );

        playerController.addEventListener(
            RemotePlayerEventType.CURRENT_TIME_CHANGED,
            (e) => {
                console.debug(`current time changed to ${e.value}`);
                state = { ...state, currentTime: e.value };
            }
        );

        playerController.addEventListener(
            RemotePlayerEventType.DURATION_CHANGED,
            (e) => {
                console.debug(`duration changed to ${e.value}`);
                state = { ...state, duration: e.value };
            }
        );

        playerController.addEventListener(
            RemotePlayerEventType.VOLUME_LEVEL_CHANGED,
            (e) => {
                console.debug(`volume level changed to ${e.value}`);
                state = { ...state, volume: e.value };
            }
        );

        playerController.addEventListener(
            RemotePlayerEventType.IS_MUTED_CHANGED,
            (e) => {
                console.debug("is muted", e.value);
                state = { ...state, isMuted: e.value };
            }
        );

        await loadMedia();

        ready = true;
    });

    async function loadMedia() {
        const context = CastContext.getInstance();

        let castSession = context.getCurrentSession();

        if (!castSession) {
            console.info("getting cast session...");

            try {
                await context.requestSession();
            } catch (e) {
                console.warn(e);
                return;
            }

            castSession = context.getCurrentSession();
        }

        const localIp = await server.getLocalIpAsync();
        const base = `${location.protocol}//${localIp}:${location.port}`;

        console.info("base path", base);
        console.info("playing video", filePath);
        console.info("subtitles path", subtitlesUrl);

        const loadRequest = createLoadRequest(base, filePath, subtitlesUrl);

        try {
            await castSession.loadMedia(loadRequest);
            console.info("media loaded");
        } catch (e) {
            console.error("failed to load media", e);
            return;
        }

        // enable subtitles
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

    function createLoadRequest(
        base: string,
        filePath: string,
        subtitlesUrl: string | null
    ) {
        const videoPath = `/video/${encodeURIComponent(filePath)}`;
        const mediaInfo = new MediaInfo(`${base}${videoPath}`);
        mediaInfo.contentType = "video/mp4";
        mediaInfo.metadata = new MovieMediaMetadata();
        mediaInfo.streamType = StreamType.BUFFERED;
        mediaInfo.textTrackStyle = new TextTrackStyle();
        mediaInfo.duration = null;
        mediaInfo.tracks = getTracks(base, subtitlesUrl);
        return new LoadRequest(mediaInfo);
    }

    function getTracks(base: string, subtitlesUrl: string | null) {
        if (subtitlesUrl) {
            const encoded = encodeURIComponent(subtitlesUrl);
            const subtitlesPath = `/subtitles/download/${encoded}`;
            const sub = new Track(1, TrackType.TEXT);
            sub.trackContentId = `${base}${subtitlesPath}`;
            sub.trackContentType = "text/vtt";
            sub.subtype = TextTrackType.SUBTITLES;
            sub.name = "English Subtitles";
            sub.language = "en-US";
            sub.customData = null;
            return [sub];
        } else {
            return [];
        }
    }

    function seek(e: CustomEvent<number>) {
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

    function mute() {
        playerController.muteOrUnmute();
    }

    function play() {
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

{#if ready}
    <VideoPlayerView
        {fileName}
        {...state}
        on:mute={mute}
        on:play={play}
        on:reload={reload}
        on:seek={seek}
        on:stop={stop}
        on:setvolume={setVolume}
    />
{/if}
