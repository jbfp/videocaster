<script lang="ts">
    import { onMount } from "svelte";
    import { decode, encode } from "./encoding";

    import FilePicker from "./states/FilePicker.svelte";
    import SubtitlesPicker from "./states/SubtitlesPicker.svelte";
    import VideoPlayer from "./states/VideoPlayer.svelte";

    let ready = false;
    let directory: string = "";
    let fileName: string | null = null;
    let subtitlesUrl: string | null = null;

    $: filePath = `${directory}__sep${fileName}`;

    $: state =
        !directory || fileName === null ? 0 : subtitlesUrl === null ? 1 : 2;

    onMount(() => {
        const args = location.pathname.slice(1).split("/").map(decode);
        directory = args[0] || "";
        fileName = args[1] || null;
        subtitlesUrl = args[2] !== undefined ? args[2] : null;
        ready = true;
    });

    function filePickerNext() {
        history.pushState(
            { directory, fileName },
            "",
            `${location.pathname}/${encode(fileName)}`
        );

        window.addEventListener("popstate", onpopstate);
    }

    function subtitlesPickerNext() {
        history.pushState(
            { directory, fileName, subtitlesUrl },
            "",
            `${location.pathname}/${encode(subtitlesUrl)}`
        );
    }

    function catchBack() {
        history.back();
    }

    function catchHome() {
        fileName = null;
        subtitlesUrl = null;
        history.pushState({ directory }, "", "/");
        window.removeEventListener("popstate", onpopstate);
    }

    function onpopstate(e: PopStateEvent) {
        directory = e.state.directory || null;
        fileName = e.state.fileName || null;
        subtitlesUrl = e.state.subtitlesUrl || null;
    }
</script>

<header>
    <h1>Videocaster</h1>
</header>

{#if ready}
    {#if state === 0}
        <FilePicker bind:directory bind:fileName on:next={filePickerNext} />
    {:else if state === 1}
        <SubtitlesPicker
            {filePath}
            bind:subtitlesUrl
            on:next={subtitlesPickerNext}
            on:home={catchHome}
        />
    {:else if state === 2}
        <VideoPlayer
            {filePath}
            {subtitlesUrl}
            on:back={catchBack}
            on:home={catchHome}
        />
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
