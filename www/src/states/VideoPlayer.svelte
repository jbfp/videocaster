<script lang="ts">
    /// <reference types="chromecast-caf-sender" />
    import { createEventDispatcher, onMount, onDestroy, tick } from "svelte";
    import * as server from "../server";
    import VideoPlayerView from "./VideoPlayerView.svelte";

    const { CastContext, CastContextEventType, SessionState } = cast.framework;

    const {
        LoadRequest,
        MediaCommand,
        MediaInfo,
        MovieMediaMetadata,
        PauseRequest,
        PlayRequest,
        SeekRequest,
        StreamType,
        TextTrackStyle,
        TextTrackType,
        Track,
        TrackType,
    } = chrome.cast.media;

    const dispatch = createEventDispatcher();

    export let filePath: string;
    export let subtitlesUrl: string;

    $: fileName = filePath.split("__sep").pop();

    const castContext = CastContext.getInstance();

    const defaultState = {
        canPause: null,
        canSeek: null,
        canChangeVolume: null,
        currentTime: null,
        duration: null,
        isMuted: null,
        mute: null,
        pause: null,
        play: null,
        receiver: null,
        playerState: null,
        seek: null,
        setVolume: null,
        volume: null,
        volumeStepInterval: null,
        unmute: null,
    };

    let state = { ...defaultState };
    let image: string;
    let currentTimeIntervalId: number | null = null;

    onMount(async () => {
        loadFrame();

        if (!window["__isGCastApiAvailable"]) {
            console.error("chromecast not available");
            return;
        }

        window.addEventListener("beforeunload", onbeforeunload);

        castContext.addEventListener(
            CastContextEventType.SESSION_STATE_CHANGED,
            onSessionStateChanged
        );

        let castSession = castContext.getCurrentSession();

        if (!castSession) {
            console.info("getting cast session...");

            try {
                await castContext.requestSession();
            } catch (e) {
                console.warn(e);
            }

            castSession = castContext.getCurrentSession();
        }

        loadMedia(castSession);
    });

    onDestroy(() => {
        window.clearInterval(currentTimeIntervalId);

        window.removeEventListener("beforeunload", onbeforeunload);

        castContext.removeEventListener(
            CastContextEventType.SESSION_STATE_CHANGED,
            onSessionStateChanged
        );
    });

    function onbeforeunload() {
        disconnect();
    }

    async function loadFrame() {
        try {
            image = await server.getVideoFrame(filePath);
        } catch (e) {
            console.error("error loading preview frame", e);
        }
    }

    async function onSessionStateChanged(
        e: cast.framework.SessionStateEventData
    ) {
        console.debug("SESSION_STATE_CHANGED", e);

        if (e.sessionState === SessionState.SESSION_STARTED) {
            await loadMedia(e.session);
        } else {
            window.clearInterval(currentTimeIntervalId);
            currentTimeIntervalId = null;
            await tick();
            state = { ...defaultState };
        }
    }

    function sessionUpdateListener() {
        const session: chrome.cast.Session = this;
        const receiver = session.receiver;
        const volume = receiver.volume;
        // @ts-ignore
        const fixed = chrome.cast.VolumeControlType.FIXED;
        // @ts-ignore
        const canChangeVolume = volume.controlType !== fixed;
        // @ts-ignore
        const volumeStepInterval = Math.round(volume.stepInterval * 100) / 100;

        state = {
            ...state,
            canChangeVolume,
            isMuted: volume.muted,
            receiver: receiver.friendlyName,
            volume: volume.level,
            volumeStepInterval,

            mute: function () {
                session.setReceiverMuted(
                    true,
                    () => console.debug("muted"),
                    (error) => console.error("mute failed", error)
                );
            },

            setVolume: function (level: number) {
                session.setReceiverVolumeLevel(
                    level,
                    () => console.debug("set volume", level),
                    (error) => console.error("set volume failed", error)
                );
            },

            unmute: function () {
                session.setReceiverMuted(
                    false,
                    () => console.debug("unmuted"),
                    (error) => console.error("unmute failed", error)
                );
            },
        };
    }

    function onMedia(media: chrome.cast.media.Media) {
        if (currentTimeIntervalId) {
            window.clearInterval(currentTimeIntervalId);
        }

        currentTimeIntervalId = window.setInterval(() => {
            const currentTime = media.getEstimatedTime();
            console.debug("current time", currentTime);
            state = { ...state, currentTime };
        }, 1000);

        const updateListener = mediaUpdateListener.bind(media);
        media.addUpdateListener(updateListener);
        updateListener();
    }

    function mediaUpdateListener() {
        const media: chrome.cast.media.Media = this;

        state = {
            ...state,
            canPause: media.supportsCommand(MediaCommand.PAUSE),
            canSeek: media.supportsCommand(MediaCommand.SEEK),
            duration: media.media.duration,
            playerState: media.playerState,

            pause: function () {
                media.pause(
                    new PauseRequest(),
                    () => console.debug("paused"),
                    (error) => console.error("pause failed", error)
                );
            },

            play: function () {
                media.play(
                    new PlayRequest(),
                    () => console.debug("playing"),
                    (error) => console.error("play failed", error)
                );
            },

            seek: function (currentTime: number) {
                const request = new SeekRequest();
                request.currentTime = currentTime;
                media.seek(
                    request,
                    () => console.debug("seek", request.currentTime),
                    (error) => console.error("seek failed", error)
                );
            },
        };
    }

    async function loadMedia(castSession: cast.framework.CastSession | null) {
        if (!castSession) {
            return;
        }

        const session = castSession.getSessionObj();
        const updateListener = sessionUpdateListener.bind(session);
        session.addUpdateListener(updateListener);
        updateListener();

        const localIp = await server.getLocalIpAsync();
        const base = `${location.protocol}//${localIp}:${location.port}`;
        const videoPath = `video/${encodeURIComponent(filePath)}`;
        const mediaInfo = new MediaInfo(`${base}/${videoPath}`, "video/mp4");
        mediaInfo.duration = null;
        mediaInfo.metadata = new MovieMediaMetadata();
        mediaInfo.streamType = StreamType.BUFFERED;
        mediaInfo.tracks = [];
        mediaInfo.textTrackStyle = new TextTrackStyle();
        mediaInfo.textTrackStyle.backgroundColor = "#000000CC";

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
            mediaInfo.tracks.push(sub);
        }

        const loadRequest = new LoadRequest(mediaInfo);

        // activate first, if any, subtitles track
        loadRequest.activeTrackIds = mediaInfo.tracks
            .slice(0, 1)
            .map((track) => track.trackId);

        session.loadMedia(loadRequest, onMedia, function (error) {
            console.error("failed to load media", error);
        });
    }

    function disconnect() {
        const castSession = castContext.getCurrentSession();

        if (castSession) {
            castSession.endSession(true);
        }
    }

    function home() {
        disconnect();
        dispatch("home");
    }
</script>

<VideoPlayerView {fileName} {image} {...state} on:home={home} />
