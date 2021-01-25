<script lang="ts">
    /// <reference types="chromecast-caf-sender" />
    import { createEventDispatcher, onMount, onDestroy, tick } from "svelte";
    import * as server from "../server";
    import VideoPlayerView from "./VideoPlayerView.svelte";

    export let filePath: string;
    export let subtitlesUrl: string;

    $: fileName = filePath.split("__sep").pop();

    const dispatch = createEventDispatcher();
    const goBack = () => dispatch("back");
    const goHome = () => dispatch("home");

    const defaultState = {
        canPause: null,
        canSeek: null,
        canChangeVolume: null,
        currentTime: null,
        duration: null,
        goBack,
        goHome,
        muted: null,
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
    let leaveSession: () => void;

    onMount(async () => {
        server
            .getVideoFrame(filePath)
            .then((img) => (image = img))
            .catch((error) => console.error("loading preview failed", error));

        const castContext = cast.framework.CastContext.getInstance();

        castContext.setOptions({
            autoJoinPolicy: chrome.cast.AutoJoinPolicy.TAB_AND_ORIGIN_SCOPED,
            receiverApplicationId:
                chrome.cast.media.DEFAULT_MEDIA_RECEIVER_APP_ID,
        });

        castContext.addEventListener(
            cast.framework.CastContextEventType.SESSION_STATE_CHANGED,
            sessionStateChanged
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

        await loadMedia(castSession?.getSessionObj());
    });

    onDestroy(() => {
        window.clearInterval(currentTimeIntervalId);

        window.removeEventListener("beforeunload", leaveSession);

        cast.framework.CastContext.getInstance().removeEventListener(
            cast.framework.CastContextEventType.SESSION_STATE_CHANGED,
            sessionStateChanged
        );
    });

    function sessionStateChanged(e: cast.framework.SessionStateEventData) {
        console.debug("SESSION_STATE_CHANGED", e);

        if (e.sessionState === cast.framework.SessionState.SESSION_STARTED) {
            const session = e.session.getSessionObj();

            window.addEventListener(
                "beforeunload",
                (leaveSession = function () {
                    session.leave(
                        () => console.debug("left session"),
                        (error) => console.error("leave failed", error)
                    );
                }.bind(undefined, [session]))
            );

            loadMedia(session);
        } else {
            window.removeEventListener("beforeunload", leaveSession);
            window.clearInterval(currentTimeIntervalId);
            currentTimeIntervalId = null;
            tick().then(() => (state = { ...defaultState }));
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
            muted: volume.muted,
            receiver: receiver.friendlyName,
            volume: volume.level,
            volumeStepInterval,

            goHome: function () {
                leaveSession();
                goHome();
            },

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
        console.debug("media loaded");

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
            canPause: media.supportsCommand(
                chrome.cast.media.MediaCommand.PAUSE
            ),
            canSeek: media.supportsCommand(chrome.cast.media.MediaCommand.SEEK),
            duration: media.media.duration,
            playerState: media.playerState,

            pause: function () {
                media.pause(
                    new chrome.cast.media.PauseRequest(),
                    () => console.debug("paused"),
                    (error) => console.error("pause failed", error)
                );
            },

            play: function () {
                media.play(
                    new chrome.cast.media.PlayRequest(),
                    () => console.debug("playing"),
                    (error) => console.error("play failed", error)
                );
            },

            seek: function (currentTime: number) {
                const request = new chrome.cast.media.SeekRequest();
                request.currentTime = currentTime;
                media.seek(
                    request,
                    () => console.debug("seek", request.currentTime),
                    (error) => console.error("seek failed", error)
                );
            },
        };
    }

    async function loadMedia(session: chrome.cast.Session | null) {
        if (!session) {
            return;
        }

        const updateListener = sessionUpdateListener.bind(session);
        session.addUpdateListener(updateListener);
        updateListener();

        const localIp = await server.getLocalIpAsync();
        const base = `${location.protocol}//${localIp}:${location.port}`;
        const videoPath = `video/${encodeURIComponent(filePath)}`;
        const contentId = `${base}/${videoPath}`;
        const contentType = "video/mp4";
        const mediaInfo = new chrome.cast.media.MediaInfo(
            contentId,
            contentType
        );
        mediaInfo.duration = null;
        mediaInfo.metadata = new chrome.cast.media.MovieMediaMetadata();
        mediaInfo.streamType = chrome.cast.media.StreamType.BUFFERED;
        mediaInfo.tracks = [];
        mediaInfo.textTrackStyle = new chrome.cast.media.TextTrackStyle();
        mediaInfo.textTrackStyle.backgroundColor = "#000000CC";

        if (subtitlesUrl) {
            const encoded = encodeURIComponent(subtitlesUrl);
            const subtitlesPath = `/subtitles/download/${encoded}`;
            const trackId = 1;
            const sub = new chrome.cast.media.Track(
                trackId,
                chrome.cast.media.TrackType.TEXT
            );
            sub.trackContentId = `${base}${subtitlesPath}`;
            sub.trackContentType = "text/vtt";
            sub.subtype = chrome.cast.media.TextTrackType.SUBTITLES;
            sub.name = "English Subtitles";
            sub.language = "en-US";
            sub.customData = null;
            mediaInfo.tracks.push(sub);
        }

        const loadRequest = new chrome.cast.media.LoadRequest(mediaInfo);

        // activate first, if any, subtitles track
        loadRequest.activeTrackIds = mediaInfo.tracks
            .slice(0, 1)
            .map((track) => track.trackId);

        session.loadMedia(loadRequest, onMedia, function (error) {
            console.error("failed to load media", error);
        });
    }
</script>

<VideoPlayerView {fileName} {image} {...state} />
