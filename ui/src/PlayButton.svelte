<script lang="ts">
    import { createEventDispatcher } from "svelte";

    enum PlayerState {
        IDLE = "IDLE",
        PLAYING = "PLAYING",
        PAUSED = "PAUSED",
        BUFFERING = "BUFFERING",
    }

    export let playerState: string;

    $: text =
        playerState === PlayerState.PLAYING
            ? "Pause"
            : playerState === PlayerState.PAUSED
            ? "Play"
            : "";

    $: disabled =
        playerState === PlayerState.IDLE ||
        playerState === PlayerState.BUFFERING;

    const dispatch = createEventDispatcher();

    function playOrPause() {
        dispatch("playOrPause");
    }
</script>

<style>
    button {
        width: 64px;
    }
</style>

<button {disabled} on:click={playOrPause}> {text} </button>
