<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import Range from "../Range.svelte";

    export let fileName: string;
    export let playerState: string;
    export let volume: number;
    export let isMuted: boolean;
    export let canSeek: boolean;
    export let currentTime: number;
    export let duration: number;

    $: muteBtnText = isMuted ? "Un-mute" : "Mute";

    $: playBtnText =
        playerState === "PLAYING"
            ? "Pause"
            : playerState === "PAUSED"
            ? "Play"
            : playerState === "BUFFERING"
            ? "Buffering"
            : "‎Loading...";

    $: playBtnDisabled =
        !playerState ||
        playerState === "IDLE" ||
        playerState === "BUFFERING";

    const volumeFormatter = (x: number) => `${Math.trunc(100 * x)}%`;

    const timeFormatter = (x: number)  => {
        const ss = duration - x;
        const h = 60 * 60;
        const h10 = 10 * h;
        const [from, length] = ss >= h10 ? [11, 8] : ss >= h ? [12, 7] : [14, 5];
        const time = new Date(ss * 1000).toISOString().substr(from, length);
        return `-${time}`;
    };

    const dispatch = createEventDispatcher();

    function play() {
        dispatch("play");
    }

    function mute() {
        dispatch("mute");
    }

    function setVolume(e: CustomEvent<number>) {
        dispatch("setvolume", e.detail);
    }

    function reload() {
        dispatch("reload");
    }

    function stop() {
        dispatch("stop");
    }

    function seek(e: CustomEvent<number>) {
        dispatch("seek", e.detail);
    }
</script>

<h2>
    <span>Now Playing </span>
    <strong class="file-name">{fileName}</strong>
</h2>

<div class="fill" />

<div class="flex-horizontal">
    <button on:click={play} disabled={playBtnDisabled}>{playBtnText}</button>
    <button on:click={mute}>{muteBtnText}</button>
    <Range
        value={volume}
        min={0}
        max={1}
        step={0.01}
        on:change={setVolume}
        formatter={volumeFormatter}
    />
    <button on:click={reload}>Reload</button>
    <button on:click={stop}> Stop </button>
</div>

<Range
    value={currentTime}
    min={0}
    max={duration}
    step={1}
    disabled={!canSeek}
    formatter={timeFormatter}
    on:change={seek}
/>

<style>
    .file-name {
        color: -webkit-activelink;
    }
</style>
