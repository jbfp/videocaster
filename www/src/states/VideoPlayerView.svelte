<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import IconButton from "../IconButton.svelte";
    import Range from "../Range.svelte";

    export let fileName: string;
    export let receiver: string;
    export let image: string;
    export let playerState: string;
    export let volume: number;
    export let isMuted: boolean;
    export let canSeek: boolean;
    export let currentTime: number;
    export let duration: number;

    $: playIcon = playerState === "PLAYING" ? "pause" : "play_arrow";
    $: muteIcon = isMuted ? "volume_off" : "volume_up";

    $: playBtnDisabled =
        !playerState || playerState === "IDLE" || playerState === "BUFFERING";

    const timeFormatter = (x: number) => {
        const h = 60 * 60;
        const h10 = 10 * h;
        const [from, length] = x >= h10 ? [11, 8] : x >= h ? [12, 7] : [14, 5];
        return new Date(x * 1000).toISOString().substr(from, length);
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

    function home() {
        dispatch("home");
    }

    function seek(e: CustomEvent<number>) {
        dispatch("seek", e.detail);
    }
</script>

<h2>
    <span>Now Playing </span>
    <strong id="file-name">{fileName}</strong>

    {#if receiver}
        <span>on {receiver}</span>
    {/if}
</h2>

<img class="fill" alt="" src={image} draggable="false" />

<div class="flex flex-horizontal">
    <IconButton
        icon={"exit_to_app"}
        title="Stop video and go to start"
        on:click={home}
    />

    <google-cast-launcher />

    <div class="separator disabled" />

    <IconButton icon={playIcon} on:click={play} disabled={playBtnDisabled} />

    <div class="fill">
        {#if playerState === "BUFFERING"}
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
                on:change={seek}
            />
        {/if}
    </div>

    <IconButton icon={muteIcon} on:click={mute} />

    <div id="volume">
        <Range
            min={0}
            value={volume}
            max={1}
            step={0.01}
            showvalue={false}
            on:change={setVolume}
        />
    </div>
</div>

<style>
    google-cast-launcher {
        width: 24px;
    }

    #file-name {
        color: -webkit-activelink;
    }

    img {
        margin: 1em -1em;
        object-fit: cover;
        height: 0;
    }

    #volume {
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
