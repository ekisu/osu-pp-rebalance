const profileTab = () => {
    document.getElementById("profile_form").className = "tab-content is-active";
    document.getElementById("profile_tab").className = "is-active";
    document.getElementById("beatmap_form").className = "tab-content";
    document.getElementById("beatmap_tab").className = "";
}

const beatmapTab = () => {
    document.getElementById("profile_form").className = "tab-content";
    document.getElementById("profile_tab").className = "";
    document.getElementById("beatmap_form").className = "tab-content is-active";
    document.getElementById("beatmap_tab").className = "is-active";
}

const stopProfileLoadingAnimation = () => document.getElementById("button").className = document.getElementById("button").className.replace(" is-loading", "");

const checkPPRequest = async (user, last_status, last_queue_pos) => {
    let resp = await fetch("/pp_check?user=" + encodeURIComponent(user));

    let json = await resp.json();
    let status = json["status"];

    console.log(json);

    if (last_status != status) {
        if (status == "pending") {
            toastr.info("In queue... (" + json["pos"] + " people ahead)");
            last_queue_pos = json["pos"];
        } else if (status == "calculating") {
            toastr.info("Calculating new PP...", "", {timeOut: 0, extendedTimeOut: 0});
        } else if (status == "error") {
            toastr.error("Error while calculating", "", {timeOut: 0, extendedTimeOut: 0});
        }
    } else {
        if (status == "pending" && last_queue_pos != json["pos"]) {
            toastr.info("In queue... (" + json["pos"] + " people ahead)");
            last_queue_pos = json["pos"];
        }
    }

    if (status != "done" && status != "error") {
        setTimeout(() => checkPPRequest(user, status, last_queue_pos), 2000);
    } else if (status == "done") {
        stopProfileLoadingAnimation();
        window.location.href = "/pp?user=" + encodeURIComponent(user);
    } else if (status == "error") {
        stopProfileLoadingAnimation();
    }
}

const requestPPCalc = async (user, force) => {
    let resp = await fetch("/pp_request?user=" + encodeURIComponent(user) + "&force=" + encodeURIComponent(force));

    let json = await resp.json()
    let status = json["status"];
    console.log("status");

    if (status == "done") {
        stopProfileLoadingAnimation();

        window.location.href = "/pp?user=" + encodeURIComponent(user);
    } else if (status == "cant_force") {
        stopProfileLoadingAnimation();

        toastr.error("Can't force recalculation for this user yet. "
            + json["remaining"] + " seconds until force is available.", "", {timeOut: 0, extendedTimeOut: 0});
    } else {
        console.log("pending, now waiting...");

        checkPPRequest(user);
    }
}

const fieldValueById = (id) => document.getElementById(id).value;

const onProfileFormSubmit = () => {
    let user = fieldValueById("user");
    let force = document.getElementById("force").checked;
    document.getElementById("button").className += " is-loading";

    requestPPCalc(user, force);

    return false;
}

const beatmapRegexes = [
    /^.*osu[.]ppy[.]sh\/beatmapsets\/\d+#osu\/(\d+).*$/,
    /^.*osu[.]ppy[.]sh\/b\/(\d+).*$/
];

const availableMods = ["HD", "HR", "DT", "FL", "NF", "EZ", "HT", "SO"];

const sendBeatmapRequest = async () => {
    let beatmap = fieldValueById("beatmap");
    
    let accPctField = fieldValueById("acc_pct");
    let goodField = fieldValueById("good");
    let mehField = fieldValueById("meh");

    let comboField = fieldValueById("combo");
    let missesField = fieldValueById("misses");
    
    let modsField = fieldValueById("mods");

    let simulation_params = {};
    let beatmap_id = parseInt(beatmap);
    // scream
    if (Object.is(beatmap_id, NaN)) { // because, apparently, NaN === NaN is false. xd
        let match = beatmapRegexes.map((r) => beatmap.match(r)).find((v) => v !== null)

        if (!match) {
            toastr.error("Beatmap field is invalid.");
            return false;
        }

        beatmap_id = parseInt(match[1])
        if (Object.is(beatmap_id, NaN)) {
            toastr.error("Beatmap field is invalid.");
            return false;
        }
    }

    let accPct = parseFloat(accPctField.replace(",", "."));
    let good = parseInt(goodField);
    let meh = parseInt(mehField);

    if ([accPct, good, meh].every((x) => Object.is(x, NaN))) {
        toastr.error("Fill either accuracy (%) or number of 300s and 100s!");
        return false;
    } else if (!Object.is(accPct, NaN)) {
        if (accPct < 0 || accPct > 100) {
            toastr.error("Accuracy (%) should be between 0 and 100.");
            return false;
        }
        simulation_params.accuracy = accPct;
    } else {
        simulation_params.accuracy = {
            good: good || 0,
            meh: meh || 0
        };
    }

    let combo = parseInt(comboField);
    if (!Object.is(combo, NaN)) {
        simulation_params.combo = combo;
    }

    let misses = parseInt(missesField);
    if (!Object.is(misses, NaN)) {
        simulation_params.misses = misses;
    }

    simulation_params.mods = modsField.toUpperCase().split(",").filter((v) => v != "");
    if (!simulation_params.mods.every((mod) => availableMods.lastIndexOf(mod) !== -1)) {
        toastr.error("Mods field is invalid.");
        return false;
    }

    let res = await fetch("/simulate", {
        method: "post",
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            beatmap_id: beatmap_id,
            params: simulation_params
        })
    });

    let json = await res.json();
    if (json.status == "error") {
        toastr.error("Error while calculating beatmap pp");
        return false;
    }

    showBeatmapCalcResult(json.results);
    return false;
}

const setInnerById = (id, val) => document.getElementById(id).innerHTML = val;

const showBeatmapCalcResult = (data) => {
    setInnerById("beatmap-results-name", data.beatmap_info);
    let mods = "";
    if (data.mods.length > 0) {
        mods = data.mods.join(",");
    } else {
        mods = "None";
    }

    setInnerById("beatmap-results-accuracy", data.play_info.accuracy.toFixed(2));
    setInnerById("beatmap-results-mods", mods);
    setInnerById("beatmap-results-combo", data.play_info.combo);
    setInnerById("beatmap-results-max-combo", data.play_info.max_combo);
    setInnerById("beatmap-results-pp", data.pp.toFixed(2));

    document.getElementById("beatmap-results").className = "modal is-active";
}

const hideBeatmapResults = () => {
    document.getElementById("beatmap-results").className = "modal";
}

const onBeatmapFormSubmit = () => {
    sendBeatmapRequest();

    return false;
}
