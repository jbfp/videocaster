<script lang="ts">
    import { createEventDispatcher, onMount } from "svelte";
    import type { Subtitle } from "../server";
    import * as server from "../server";
    import IconButton from "../IconButton.svelte";

    export let filePath: string;
    export let subtitlesUrl: string | null = null;

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

    onMount(async () => {
        const fileName = filePath.split("__sep").pop();
        const result = regex.exec(fileName);

        console.debug("regex test", result);

        if (result) {
            title = result[1].trim();
            season = result[2];
            episode = result[3];
        } else {
            title = fileName;
        }

        await search();
    });

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
        dispatch("next");
    }

    function skip() {
        subtitlesUrl = "";
        dispatch("next");
    }

    function home() {
        dispatch("home");
    }
</script>

<h2>Select Subtitles <small>(optional)</small></h2>

<div class="flex flex-vertical">
    <div class="flex flex-horizontal">
        <span class="muted">Selected video:</span>
        <code>{filePath}</code>
    </div>

    <div class="flex flex-horizontal">
        <label for="title">Title:</label>
        <input
            id="title"
            type="text"
            bind:value={title}
            placeholder="The Mundaforian"
        />
    </div>

    <div class="flex flex-horizontal">
        <label for="season">Season:</label>
        <input id="season" type="text" bind:value={season} placeholder="02" />
    </div>

    <div class="flex flex-horizontal">
        <label for="episode">Episode:</label>
        <input id="episode" type="text" bind:value={episode} placeholder="03" />
    </div>

    <div class="flex flex-horizontal">
        <button disabled={loading} on:click={search}>Search</button>

        {#if loading}
            <span class="muted">Searching...</span>
        {:else if subtitlesByPath && subtitlesByMetadata}
            <span class="muted">{numSubtitles} subtitles found</span>
        {/if}
    </div>
</div>

<select bind:value={selectedSubtitles} disabled={loading} size={10000}>
    {#each subtitlesByPath as subtitle}
        <option value={subtitle}>{subtitle.name} (Best fit)</option>
    {/each}

    {#each subtitlesByMetadata as subtitle}
        <option value={subtitle}>{subtitle.name}</option>
    {/each}
</select>

<div class="flex flex-horizontal">
    <button on:click={next} disabled={nextDisabled}>Next</button>
    <button on:click={skip}>Skip</button>
    <IconButton icon={"home"} title="Go to start" on:click={home} />

    {#if selectedSubtitles}
        <span>
            Selected subtitles: <code>{selectedSubtitles?.name}</code>
        </span>
    {/if}
</div>

<style>
    input {
        width: 250px;
    }

    label {
        display: block;
        width: 60px;
    }

    option {
        padding: 5px 0;
    }

    select {
        margin: 0 calc(-1em - 3px);
        overflow-y: auto;
        text-indent: 9px;
        width: auto;
    }

    small {
        color: #666;
    }
</style>
