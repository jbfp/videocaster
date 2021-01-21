<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { Subtitle } from "../server";
    import * as server from "../server";

    export let filePath: string;
    export let subtitlesUrl: string | null = null;

    $: {
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
    }

    const dispatch = createEventDispatcher();

    let loading = false;

    let title: string;
    let season: string | null;
    let episode: string | null;

    let subtitlesByPath: Subtitle[] = [];
    let subtitlesByMetadata: Subtitle[] = [];
    let selectedSubtitles: Subtitle;
    $: numSubtitles = subtitlesByPath.length + subtitlesByMetadata.length;
    $: nextDisabled = loading || !selectedSubtitles;

    const regex = /(.+)[sS](\d{1,2})[eE](\d{1,2}).*/;

    async function search() {
        loading = true;

        try {
            [subtitlesByPath, subtitlesByMetadata] = await Promise.all([
                server.searchSubsByPath(filePath),
                server.searchSubsByMetadataAsync(title, season, episode),
            ]);
        } finally {
            loading = false;
        }

        if (subtitlesByPath.length > 0) {
            selectedSubtitles = subtitlesByPath[0];
        } else if (subtitlesByMetadata.length > 0) {
            selectedSubtitles = subtitlesByMetadata[0];
        } else {
            selectedSubtitles = undefined;
        }
    }

    function next() {
        subtitlesUrl = selectedSubtitles.url;
        dispatch("select");
    }

    function skip() {
        subtitlesUrl = "";
        dispatch("select");
    }
</script>

<h2>Select Subtitles <small>(optional)</small></h2>

<div class="flex-horizontal">
    <label for="title">Title:</label>
    <input
        id="title"
        type="text"
        bind:value={title}
        placeholder="The Mundaforian"
    />
</div>

<div class="flex-horizontal">
    <label for="season">Season:</label>
    <input id="season" type="text" bind:value={season} placeholder="02" />
</div>

<div class="flex-horizontal">
    <label for="episode">Episode:</label>
    <input id="episode" type="text" bind:value={episode} placeholder="03" />
</div>

<div class="flex-horizontal">
    <button disabled={loading} on:click={search}>Search</button>

    {#if loading}
        <span>Searching...</span>
    {:else if subtitlesByPath && subtitlesByMetadata}
        <span>{numSubtitles} subtitles found.</span>

        {#if numSubtitles === 0}
            <span>Perhaps the title is missing an apostrophe or other special characters.</span>
        {/if}
    {/if}
</div>

<select bind:value={selectedSubtitles} disabled={loading} size={10000}>
    {#each subtitlesByPath as subtitle}
        <option value={subtitle}>{subtitle.name} (Best fit)</option>
    {/each}

    {#each subtitlesByMetadata as subtitle}
        <option value={subtitle}>{subtitle.name}</option>
    {/each}
</select>

<div class="flex-horizontal">
    <button on:click={next} disabled={nextDisabled}>Next</button>
    <button on:click={skip}>Skip</button>
</div>

{#if selectedSubtitles}
    <div>
        Selected subtitles: <code>{selectedSubtitles?.name}</code>
    </div>
{/if}

<style>
    label {
        display: block;
        width: 60px;
    }

    input[type="text"] {
        width: 250px;
    }

    select {
        flex: 1 0;
    }

    small {
        color: #666;
    }
</style>
