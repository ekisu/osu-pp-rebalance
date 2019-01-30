const checkPPRequest = async (user, last_status) => {
    let resp = await fetch("/pp_check?user=" + encodeURIComponent(user));

    let status = await resp.text();

    console.log(status);

    if (last_status != status) {
        if (status == "pending") {
            toastr.info("In queue...");
        } else if (status == "calculating") {
            toastr.info("Calculating new PP...");
        } else if (status == "error") {
            toastr.error("Error while calculating");
        }
    }

    if (status != "done" && status != "error") {
        setTimeout(() => checkPPRequest(user, status), 2000);
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
