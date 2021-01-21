<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let title: string | undefined = undefined;
    export let disabled: boolean | undefined = undefined;
    export let value: number;
    export let min: number;
    export let max: number;
    export let step: number;
    export let showmin: boolean = false;
    export let showvalue: boolean = true;
    export let showmax: boolean = false;
    export let formatter: (x: number) => string | undefined = undefined;

    let temp: number;
    let changing = false;

    $: minText = showmin ? (formatter ? formatter(min) : `${min}`) : "";
    $: valueText = showvalue ? (formatter ? formatter(temp) : `${temp}`) : "";
    $: maxText = showmax ? (formatter ? formatter(max) : `${max}`) : "";

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

<div class="flex flex-horizontal">
    {#if showmin}
        <div class="muted">{minText}</div>
    {/if}
    <div class="flex fill">
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
    </div>
    {#if showvalue}
        <div>{valueText}</div>
    {/if}
    {#if showvalue && showmax}
        <div class="muted">/</div>
    {/if}
    {#if showmax}
        <div class="muted">{maxText}</div>
    {/if}
</div>

<style>
    input[type="range"] {
        width: 100%;
    }
</style>
