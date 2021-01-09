<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let canSeek: boolean;
    export let currentTime: number;
    export let duration: number;

    $: currentTimeStr = currentTime ? secondsToString(currentTime) : ":";
    $: durationStr = duration ? secondsToString(duration) : ":";

    const dispatch = createEventDispatcher();

    function seek(e: Event) {
        dispatch("seek", Number.parseFloat((<HTMLInputElement>e.target).value));
    }

    function finishSeek(e: Event) {
        dispatch(
            "finishSeek",
            Number.parseFloat((<HTMLInputElement>e.target).value)
        );
    }

    function secondsToString(ss: number) {
        return new Date(ss * 1000).toISOString().substr(11, 8);
    }
</script>

<style>
    input {
        width: 500px;
    }
</style>

<input
    type="range"
    step="1"
    disabled={!canSeek}
    value={currentTime}
    max={duration}
    on:input={seek}
    on:change={finishSeek} />

{currentTimeStr}&nbsp;/&nbsp;{durationStr}
