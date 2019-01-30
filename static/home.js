const checkPPRequest = async (user, last_status, last_queue_pos) => {
    let resp = await fetch("/pp_check?user=" + encodeURIComponent(user));

    let json = await resp.json();
    let status = json["status"];

    console.log(json);

    if (last_status != status) {
        if (status == "pending") {
            toastr.info("In queue... (" + json["pos"] + ")");
            last_queue_pos = json["pos"];
        } else if (status == "calculating") {
            toastr.info("Calculating new PP...");
        } else if (status == "error") {
            toastr.error("Error while calculating");
        }
    } else {
        if (status == "pending" && last_queue_pos != json["pos"]) {
            toastr.info("In queue... (" + json["pos"] + ")");
            last_queue_pos = json["pos"];
        }
    }

    if (status != "done" && status != "error") {
        setTimeout(() => checkPPRequest(user, status, last_queue_pos), 2000);
    } else if (status == "done") {
        document.getElementById("button").className = document.getElementById("button").className.replace(" is-loading", "");
        window.location.href = "/pp?user=" + encodeURIComponent(user);
    } else if (status == "error") {
        document.getElementById("button").className = document.getElementById("button").className.replace(" is-loading", "");
    }
}

const requestPPCalc = async (user) => {
    let resp = await fetch("/pp_request?user=" + encodeURIComponent(user));

    let status = (await resp.json())["status"];
    console.log("status");

    if (status == "done") {
        document.getElementById("button").className = document.getElementById("button").className.replace(" is-loading", "");

        window.location.href = "/pp?user=" + encodeURIComponent(user);
    } else {
        console.log("pending, now waiting...");

        checkPPRequest(user);
    }
}

const onFormSubmit = () => {
    let user = document.getElementById("user").value;
    document.getElementById("button").className += " is-loading";

    requestPPCalc(user);

    return false;
}
