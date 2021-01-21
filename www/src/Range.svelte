<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let title: string | undefined;
    export let disabled: boolean;
    export let value: number;
    export let min: number;
    export let max: number;
    export let step: number;
    export let formatter: (x: number) => string | undefined;

    let temp: number;
    let changing = false;

    $: text = formatter ? formatter(temp) : `${temp}`;

    $: if (!changing) {
        temp = value;
    }

    const dispatch = createEventDispatcher();

    function input() {
        changing = true;
    }

    function change() {
        changing = false;
        dispatch("change", temp);
    }
</script>

<div class="flex-horizontal">
    <input
        type="range"
        {title}
        {min}
        {max}
        {step}
        {disabled}
        bind:value={temp}
        on:input={input}
        on:change={change}
    />
    <div>{text}</div>
</div>

<style>
    input[type="range"] {
        width: 100%;
    }
</style>
