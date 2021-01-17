<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let volume: number;

    const dispatch = createEventDispatcher();

    function setVolume(e: Event) {
        dispatch(
            "setVolume",
            Number.parseFloat((<HTMLInputElement>e.target).value)
        );
    }
</script>

<input
    type="range"
    min="0"
    max="1"
    value={volume}
    step="0.01"
    on:change={setVolume} />
