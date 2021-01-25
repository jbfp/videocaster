<script lang="ts">
    /// <reference types="chromecast-caf-sender" />
    import IconButton from "../IconButton.svelte";
    import Range from "../Range.svelte";

    const { PlayerState } = chrome.cast.media;

    export let canChangeVolume: boolean;
    export let canPause: boolean;
    export let canSeek: boolean;
    export let currentTime: number;
    export let duration: number;
    export let fileName: string;
    export let image: string;
    export let muted: boolean;
    export let playerState: string;
    export let receiver: string;
    export let volume: number;
    export let volumeStepInterval: number;
    export let goHome: () => void;
    export let mute: () => void;
    export let pause: () => void;
    export let play: () => void;
    export let seek: (currentTime: number) => void;
    export let setVolume: (level: number) => void;
    export let unmute: () => void;

    const timeFormatter = (x: number) => {
        const h = 60 * 60;
        const h10 = 10 * h;
        const [from, length] = x >= h10 ? [11, 8] : x >= h ? [12, 7] : [14, 5];
        return new Date(x * 1000).toISOString().substr(from, length);
    };

    function onseek(e: CustomEvent<number>) {
        seek?.(e.detail);
    }

    function onsetvolume(e: CustomEvent<number>) {
        setVolume?.(e.detail);
    }
</script>

<h2>
    <span>Now Playing </span>
    <strong class="file-name">{fileName}</strong>

    {#if receiver}
        <span>on {receiver}</span>
    {/if}
</h2>

<img class="fill" alt="" src={image} draggable="false" />

<div class="flex flex-horizontal">
    <IconButton
        icon={"home"}
        title="Stop video and go to start"
        on:click={goHome}
    />

    <google-cast-launcher />

    <div class="separator disabled" />

    {#if playerState === PlayerState.PLAYING}
        <IconButton icon="pause" on:click={pause} disabled={!canPause} />
    {:else if playerState === PlayerState.PAUSED}
        <IconButton icon="play_arrow" on:click={play} />
    {:else}
        <IconButton icon="play_arrow" disabled />
    {/if}

    <div class="fill">
        {#if playerState === PlayerState.BUFFERING}
            <progress />
        {:else}
            <Range
                min={0}
                value={currentTime}
                max={duration}
                step={1}
                disabled={!canSeek}
                showvalue={true}
                showmax={true}
                formatter={timeFormatter}
                on:change={onseek}
            />
        {/if}
    </div>

    {#if muted}
        <IconButton
            icon="volume_off"
            on:click={unmute}
            disabled={!canChangeVolume}
        />
    {:else}
        <IconButton
            icon="volume_up"
            on:click={mute}
            disabled={!canChangeVolume}
        />
    {/if}

    <div class="volume">
        <Range
            min={0}
            value={volume}
            max={1}
            step={volumeStepInterval}
            disabled={!canChangeVolume}
            showvalue={false}
            on:change={onsetvolume}
        />
    </div>
</div>

<style>
    google-cast-launcher {
        width: 24px;
    }

    .file-name {
        color: -webkit-activelink;
    }

    img {
        margin: 1em -1em;
        object-fit: cover;
        height: 0;
    }

    .volume {
        width: 100px;
    }

    progress {
        width: 100%;
    }

    .separator {
        border-left: 1px solid currentColor;
        height: 100%;
    }

    .separator::before {
        content: "";
    }
</style>
