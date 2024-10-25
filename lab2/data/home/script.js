//SAVE USER`S NAME
async function naming(name){
    const response = await fetch("http://217.71.129.139:4798/api/session_info", {
        method: "GET",
    });
    
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    const data=await response.json();
    let header=document.getElementById("name");
    let par=document.getElementById("hash");
    console.log(data);
    if (data && data.firstName) {
        header.innerHTML = "Hello, " + data.name;
    } else {
        header.innerHTML = `Hello, ${data.name}`;
    }
    if (data && data.hash){
        par.innerHTML="Your hash:"+ data.hash.substr(0,5);
    }
}
//SUBMIT OPERATION
async function submitbtn() {
    let operation = +document.getElementById("operations").value;
    let val1 = +document.getElementById("1").value;
    let val2 = +document.getElementById("2").value;
    let result;
    if (val1!="" && val2!=""){
        const response = await fetch("http://217.71.129.139:4798/api/calculate", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                num1: val1,
                num2: val2,
                operator_id: operation,
                result: null,
            }),
        });

        if (!response.ok) {
            throw new Error(`Response status: ${response.status}`);
        }

        let label = document.getElementById("res");

        const json = await response.json();
        console.log(json);
        label.innerHTML = "Result: " + json.result;
    }
    else{
        alert("Calculator poles are empty!");
    }
    //saveOperation(`${val1} ${operation} ${val2} = ${result}`);
}
//HISTORY
async function saveOperation(operation) {
        const response = await fetch("http://217.71.129.139:4798/api/history", {
            method: "GET",
        });
        
        if (!response.ok) {
            throw new Error(`Response status: ${response.status}`);
        }

        const history = await response.json();
        const trimmed = `${json.num1} ${json.operator_id} ${json.num2}${json.result}`;
        console.log(history);
}

//LOGIN
async function login(){

    let firstName=document.getElementById("firstName").value;
    let password=document.getElementById("password").value;
    let label =document.getElementById("message");
    const response = await fetch("http://217.71.129.139:4798/api/login", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            name: firstName,
            password: password,
        }),
    });
    if (!response.ok) {
        
        label.innerHTML="Inccorrect password or login!";
        
    }
    else{
        let name=document.getElementById("firstName").value;
        alert(`Sign in was completed succesfully\nHello, ${name}`);
        
        window.location.href="/";
    }
    console.log(response);
}
document.addEventListener("DOMContentLoaded", naming);

//REGISTRATION
async function reg(){
    document.getElementById('registrationForm').addEventListener('submit', async function(event) {
        event.preventDefault();

        const firstName = document.getElementById('firstName').value;
        const password = document.getElementById('password').value;

        try {
            const response = await fetch("http://217.71.129.139:4798/api/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    name: firstName,
                    password: password
                }),
            });

            if (!response.ok) {
                throw new Error(`Server error: ${response.status}`);
            }
            window.location.href = '/';
        } catch (error) {
            console.error('Error during registration:', error);
            alert('Failed to register. Please try again.');
        }
    });
}
async function logout() {
    const response = await fetch("http://217.71.129.139:4798/api/logout", {
        method: "GET",
    });
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    else{
        window.location.href="/";
    }
}

//THEME
document.getElementById('theme').addEventListener('click', function() {
    const currentTheme = document.body.className;
    if (currentTheme === 'light-theme') {
        document.body.className = 'dark-theme';
    } else {
        document.body.className = 'light-theme';
    }
});
//PROFILE IMAGE
/*async function image(){
    const response = await fetch("http://217.71.129.139:4798/api/image", {
        method: "GET",
    });
    if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
    }
    const data = await response.json();
    let image=document.getElementById("im");
    if (data && data.image){
        image.innerHTML=data.image;
    }
}*/
    