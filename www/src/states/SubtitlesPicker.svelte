<script lang="ts">
    import { onMount, createEventDispatcher } from "svelte";
    import type { Subtitle } from "../server";
    import * as server from "../server";

    export let filePath: string;

    const dispatch = createEventDispatcher();

    let title: string;
    let season: string | null;
    let episode: string | null;

    let subtitlesByPath: Subtitle[] = [];
    let subtitlesByMetadata: Subtitle[] = [];
    let selectedSubtitles: Subtitle | null | undefined;
    $: disabled = typeof selectedSubtitles === "undefined";

    const regex = /(.+)[sS](\d{1,2})[eE](\d{1,2}).*/;

    onMount(() => {
        const separator = "__sep"; // replaced at compile time
        const split = filePath.split(separator);
        const fileName = split[split.length - 1];
        const result = regex.exec(fileName);

        console.info("regex test", result);

        if (result) {
            title = result[1].trim();
            season = result[2];
            episode = result[3];
        } else {
            title = fileName;
        }

        search();
    });

    async function search() {
        [subtitlesByPath, subtitlesByMetadata] = await Promise.all([
            server.searchSubsByPath(filePath),
            server.searchSubsByMetadataAsync(title, season, episode),
        ]);

        if (subtitlesByPath.length > 0) {
            selectedSubtitles = subtitlesByPath[0];
        } else if (subtitlesByMetadata.length > 0) {
            selectedSubtitles = subtitlesByMetadata[0];
        } else {
            selectedSubtitles = undefined;
        }
    }

    function next() {
        dispatch("subtitleUrlSelected", selectedSubtitles?.url);
    }
</script>

<h2>Select Subtitles</h2>

<div>
    <label for="title">Title:</label>
    <input
        id="title"
        type="text"
        bind:value={title}
        placeholder="The Mundaforian"
    />
</div>

<div>
    <label for="season">Season:</label>
    <input id="season" type="text" bind:value={season} placeholder="02" />
</div>

<div>
    <label for="episode">Episode:</label>
    <input id="episode" type="text" bind:value={episode} placeholder="03" />
</div>

<div>
    <button on:click={search}>Search</button>

    {#if subtitlesByPath && subtitlesByMetadata}
        {subtitlesByPath.length + subtitlesByMetadata.length || "no"} subtitles found
    {/if}
</div>

<div>
    <select bind:value={selectedSubtitles}>
        {#each subtitlesByPath as subtitle}
            <option value={subtitle}>{subtitle.name} (Best fit)</option>
        {/each}

        {#each subtitlesByMetadata as subtitle}
            <option value={subtitle}>{subtitle.name}</option>
        {/each}

        <option value={null}>without subtitles</option>
    </select>
</div>

<div>
    <button on:click={next} {disabled}>Next</button>

    {#if !disabled}
        Selected subtitles: <code
            >{selectedSubtitles?.name || "without subtitles"}</code
        >
    {/if}
</div>

<style>
    div {
        margin-bottom: 0.5em;
    }

    label {
        display: inline-block;
        width: 60px;
    }

    input[type="text"] {
        width: 250px;
    }

    select {
        max-width: 500px;
        min-width: 300px;
        width: auto;
    }
</style>
