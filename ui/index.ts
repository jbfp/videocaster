const $selectedFile = document.getElementById('selected-file') as HTMLSpanElement;
const $playerState = document.getElementById('player-state') as HTMLSpanElement;
const $start = document.getElementById('start-btn') as HTMLButtonElement;
const $play = document.getElementById('play-btn') as HTMLButtonElement;
const $mute = document.getElementById('mute-btn') as HTMLButtonElement;
const $progress = document.getElementById('progress') as HTMLInputElement;
const $volume = document.getElementById('volume') as HTMLInputElement;
const $currentTime = document.getElementById('current-time') as HTMLSpanElement;
const $duration = document.getElementById('duration') as HTMLSpanElement;
const $fileList = document.getElementById('file-list') as HTMLUListElement;
const $currentDir = document.getElementById('current-dir') as HTMLDivElement;

function secondsToString(ss: number) {
    return new Date(ss * 1000).toISOString().substr(11, 8);
}

async function chromecastMain() {
    let localIp: string | null = null;
    let currentDir: string | null = null;
    let selectedFile: string | null = null;

    async function getLocalIp() {
        let res;

        try {
            res = await fetch('/ip');
        } catch (e) {
            console.error(e);
            return;
        }

        try {
            localIp = await res.json()
        } catch (e) {
            console.error(e);
            return;
        }

        console.info('local ip', localIp);
    }

    function setCurrentDir(s: string) {
        currentDir = s;
        $currentDir.textContent = currentDir;
    }

    function setSelectedFile(s: string) {
        selectedFile = s;
        $selectedFile.textContent = selectedFile;
    }

    async function loadDir() {
        const path = currentDir ? `?path=${encodeURIComponent(currentDir)}` : '';
        const url = `/fs${path}`;

        let res;

        try {
            res = await fetch(url);
        } catch (e) {
            console.error(e);
            return;
        }

        let result;

        try {
            result = await res.json()
        } catch (e) {
            console.error(e);
            return;
        }

        setCurrentDir(result.realPath);

        while ($fileList.firstChild) {
            $fileList.removeChild($fileList.firstChild);
        }

        function appendItemElement({ isDir, name }) {
            const $child = document.createElement('li');
            const $a = document.createElement('a');
            $child.classList.add('file-list-item');
            $child.dataset.type = isDir ? 'dir' : 'file';
            $a.textContent = name;
            $a.href = `#${name}`;
            $a.addEventListener('click', async function () {
                const path = `${currentDir}/${name}`;

                if (isDir) {
                    setCurrentDir(path);
                    await loadDir();
                } else {
                    setSelectedFile(path);
                    await loadMedia();
                }
            });
            $child.appendChild($a);
            $fileList.appendChild($child);
        }

        for (const item of result.items) {
            appendItemElement(item);
        }
    }

    const context = cast.framework.CastContext.getInstance();

    context.setOptions({
        receiverApplicationId: chrome.cast.media.DEFAULT_MEDIA_RECEIVER_APP_ID,
        autoJoinPolicy: chrome.cast.AutoJoinPolicy.ORIGIN_SCOPED,
        resumeSavedSession: true,
    });

    async function loadMedia() {
        let castSession = context.getCurrentSession();

        if (!castSession) {
            console.info('getting cast session...');

            try {
                await context.requestSession();
            } catch (e) {
                return;
            }

            castSession = context.getCurrentSession();
        }

        console.info('playing video', selectedFile);

        const sub = new chrome.cast.media.Track(1, chrome.cast.media.TrackType.TEXT);
        sub.trackContentId = `http://${localIp}:8080/subtitles/${encodeURIComponent(selectedFile)}`;
        sub.trackContentType = 'text/vtt';
        sub.subtype = chrome.cast.media.TextTrackType.SUBTITLES;
        sub.name = 'English Subtitles';
        sub.language = 'en-US';
        sub.customData = null;

        const tracks = [sub];
        const mediaInfo = new chrome.cast.media.MediaInfo(`http://${localIp}:8080/video/${encodeURIComponent(selectedFile)}`);
        mediaInfo.contentType = 'video/mp4';
        mediaInfo.metadata = new chrome.cast.media.MovieMediaMetadata();
        mediaInfo.streamType = chrome.cast.media.StreamType.BUFFERED;
        mediaInfo.textTrackStyle = new chrome.cast.media.TextTrackStyle();
        mediaInfo.duration = null;
        mediaInfo.tracks = tracks;

        const loadRequest = new chrome.cast.media.LoadRequest(mediaInfo);

        try {
            await castSession.loadMedia(loadRequest);
            console.info('media loaded');
        } catch (e) {
            console.error('failed to load media', e);
            return;
        }

        let media;

        try {
            media = await castSession.getMediaSession();
            console.debug('retrieved media session');
        } catch (e) {
            console.error('failed to load media session', e);
            return;
        }

        const tracksInfoRequest = new chrome.cast.media.EditTracksInfoRequest([1]);

        try {
            await media.editTracksInfo(tracksInfoRequest);
            console.info('subtitles loaded');
        } catch (e) {
            console.warn('failed to set subtitle track', e);
        }

        $play.hidden = false;
        $progress.disabled = false;
    }

    const player = new cast.framework.RemotePlayer();
    const playerController = new cast.framework.RemotePlayerController(player);

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.PLAYER_STATE_CHANGED,
        e => {
            const ps = e.value;

            $play.disabled = false;

            switch (ps) {
                case chrome.cast.media.PlayerState.IDLE: {
                    $play.disabled = true;
                    break;
                }

                case chrome.cast.media.PlayerState.PLAYING: {
                    $play.textContent = 'Pause';
                    break;
                }

                case chrome.cast.media.PlayerState.PAUSED: {
                    $play.textContent = 'Play';
                    break;
                }

                case chrome.cast.media.PlayerState.BUFFERING: {
                    break;
                }
            }

            $playerState.textContent = ps;
        });

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.IS_MEDIA_LOADED_CHANGED,
        e => $play.disabled = !e.value);

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.CAN_SEEK_CHANGED,
        e => $progress.disabled = !e.value);

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.CURRENT_TIME_CHANGED,
        e => {
            console.debug(`progress changed to ${e.value}`);

            if (!isSeeking) {
                $progress.value = e.value;
                $currentTime.textContent = secondsToString(e.value);
            }
        });

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.DURATION_CHANGED,
        e => {
            console.debug(`duration changed to ${e.value}`);
            $progress.max = e.value;
            $duration.textContent = secondsToString(e.value);
        });

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.VOLUME_LEVEL_CHANGED,
        e => {
            console.debug(`volume level changed to ${e.value}`);
            $volume.value = e.value;
            $mute.hidden = false;
        });

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.IS_MUTED_CHANGED,
        e => $mute.textContent = e.value ? "Unmute" : "Mute");

    playerController.addEventListener(
        cast.framework.RemotePlayerEventType.TITLE_CHANGED,
        e => document.title = `Jakob's Video - ${e.value}`);

    // playerController.addEventListener(
    //     cast.framework.RemotePlayerEventType.ANY_CHANGE,
    //     e => console.log(e));

    let isSeeking = false;

    $progress.addEventListener('change', function (e) {
        isSeeking = false;

        if (!player.canSeek) {
            console.warn('cannot seek');
            return false;
        }

        const previousTime = player.currentTime;
        const newTime = Number.parseFloat((<HTMLInputElement>e.target).value);
        player.currentTime = newTime;

        try {
            playerController.seek();
            console.log(`seek from ${previousTime} to ${newTime}`);
        } catch (e) {
            console.error(e);
        }
    });

    $progress.addEventListener('input', function (e) {
        isSeeking = true;
        const ss = Number.parseFloat((<HTMLInputElement>e.target).value);
        $currentTime.textContent = secondsToString(ss);
    });

    $volume.addEventListener('change', function (e) {
        if (!player.canControlVolume) {
            console.warn('cannot control volume');
            return false;
        }

        const previousVolumeLevel = player.volumeLevel;
        const newVolumeLevel = Number.parseFloat((<HTMLInputElement>e.target).value);
        player.volumeLevel = newVolumeLevel;

        try {
            playerController.setVolumeLevel();
            console.log(`volume changed from ${previousVolumeLevel} to ${newVolumeLevel}`);
        } catch (e) {
            console.error(e);
        }
    });

    $mute.addEventListener('click', function () {
        playerController.muteOrUnmute();
    });

    $start.addEventListener('click', loadMedia);

    $play.addEventListener('click', function () {
        playerController.playOrPause();
    });

    await loadDir();
    await getLocalIp();
};

window['__onGCastApiAvailable'] = isAvailable => {
    if (isAvailable) {
        chromecastMain();
    }
};
