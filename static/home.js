const stopLoadingAnimation = () => document.getElementById("button").className = document.getElementById("button").className.replace(" is-loading", "");

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
        stopLoadingAnimation();
        window.location.href = "/pp?user=" + encodeURIComponent(user);
    } else if (status == "error") {
        stopLoadingAnimation();
    }
}

const requestPPCalc = async (user, force) => {
    let resp = await fetch("/pp_request?user=" + encodeURIComponent(user) + "&force=" + encodeURIComponent(force));

    let json = await resp.json()
    let status = json["status"];
    console.log("status");

    if (status == "done") {
        stopLoadingAnimation();

        window.location.href = "/pp?user=" + encodeURIComponent(user);
    } else if (status == "cant_force") {
        stopLoadingAnimation();

        toastr.error("Can't force recalculation for this user yet. "
            + json["remaining"] + " seconds until force is available.", "", {timeOut: 0, extendedTimeOut: 0});
    } else {
        console.log("pending, now waiting...");

        checkPPRequest(user);
    }
}

const onFormSubmit = () => {
    let user = document.getElementById("user").value;
    let force = document.getElementById("force").checked;
    document.getElementById("button").className += " is-loading";

    requestPPCalc(user, force);

    return false;
}
