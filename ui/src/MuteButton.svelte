<script lang="ts">
    import { createEventDispatcher } from 'svelte';

    export let isMuted: boolean;

    $: text = isMuted ? "Un-mute" : "Mute";

    const dispatch = createEventDispatcher();

    function muteOrUnmute() {
        dispatch("muteOrUnmute");
    }
</script>

<style>
    button {
        width: 72px;
    }
</style>

<button on:click={muteOrUnmute}>
    {text}
</button>
