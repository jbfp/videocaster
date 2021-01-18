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
            : playerState === PlayerState.BUFFERING
            ? "Buffering"
            : "‎";

    $: disabled =
        !playerState ||
        playerState === PlayerState.IDLE ||
        playerState === PlayerState.BUFFERING;

    const dispatch = createEventDispatcher();

    function playOrPause() {
        dispatch("playOrPause");
    }
</script>

<button {disabled} on:click={playOrPause}>{text}</button>

