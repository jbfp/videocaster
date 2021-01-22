<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { decode, encode } from "./encoding";

    import FilePicker from "./states/FilePicker.svelte";
    import SubtitlesPicker from "./states/SubtitlesPicker.svelte";
    import VideoPlayer from "./states/VideoPlayer.svelte";

    let ready = false;
    let directory: string | undefined;
    let fileName: string | undefined;
    let subtitlesUrl: string | undefined;

    $: filePath = `${directory}__sep${fileName}`;

    $: state =
        directory === undefined || directory === null
            ? 0
            : fileName === undefined || fileName === null
            ? 0
            : subtitlesUrl === undefined || subtitlesUrl === null
            ? 1
            : 2;

    onMount(() => {
        parsePath();
        window.addEventListener("popstate", parsePath);
        ready = true;
    });

    onDestroy(() => window.removeEventListener("popstate", parsePath));

    function parsePath() {
        try {
            [directory, fileName, subtitlesUrl] = location.pathname
                .slice(1)
                .split("/")
                .map(decode);
        } catch (e) {
            console.error(e);
        }
    }

    function catchFileNameSelected() {
        history.pushState("", "", `${location.pathname}/${encode(fileName)}`);
    }

    function catchSubtitleUrlSelected() {
        history.pushState(
            "",
            "",
            `${location.pathname}/${encode(subtitlesUrl)}`
        );
    }

    function catchStop() {
        directory = null;
        fileName = null;
        subtitlesUrl = null;
        history.pushState("", "", "/");
    }
</script>

<header>
    <h1 class="flex-horizontal">Videocaster</h1>
</header>

{#if ready}
    {#if state === 0}
        <FilePicker
            bind:directory
            bind:fileName
            on:select={catchFileNameSelected}
        />
    {:else if state === 1}
        <SubtitlesPicker
            {filePath}
            bind:subtitlesUrl
            on:select={catchSubtitleUrlSelected}
        />
    {:else if state === 2}
        <VideoPlayer {filePath} {subtitlesUrl} on:stop={catchStop} />
    {/if}
{/if}

<footer>
    <em>
        Subtitles provided by <a
            href="https://www.opensubtitles.org"
            target="_blank">OpenSubtitles.org</a
        >
        | Made by <a href="https://github.com/jbfp/" target="_blank">jbfp</a>
        |
        <a href="https://github.com/jbfp/videocaster" target="_blank"
            >Source Code</a
        >
    </em>
</footer>

<style>
    em {
        color: #666666;
        display: inline-block;
        font-size: small;
    }
</style>
