<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import { decode, encode } from "./encoding";

    import FilePicker from "./states/FilePicker.svelte";
    import SubtitlesPicker from "./states/SubtitlesPicker.svelte";
    import VideoPlayer from "./states/VideoPlayer.svelte";

    let ready = false;
    let directory: string | null = null;
    let fileName: string | null = null;
    let subtitlesUrl: string | null = null;

    $: filePath = `${directory}__sep${fileName}`;

    $: state =
        directory === null || fileName === null
            ? 0
            : subtitlesUrl === null
            ? 1
            : 2;

    onMount(() => {
        parsePath();
        window.addEventListener("popstate", parsePath);
        ready = true;
    });

    onDestroy(() => window.removeEventListener("popstate", parsePath));

    function parsePath() {
        const args = location.pathname.slice(1).split("/").map(decode);
        directory = args[0] || null;
        fileName = args[1] || null;
        subtitlesUrl = args[2] || null;
    }

    function filePickerNext() {
        history.pushState("", "", `${location.pathname}/${encode(fileName)}`);
    }

    function subtitlesPickerNext() {
        history.pushState(
            "",
            "",
            `${location.pathname}/${encode(subtitlesUrl)}`
        );
    }

    function catchStop() {
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
        <FilePicker bind:directory bind:fileName on:next={filePickerNext} />
    {:else if state === 1}
        <SubtitlesPicker
            {filePath}
            bind:subtitlesUrl
            on:next={subtitlesPickerNext}
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
