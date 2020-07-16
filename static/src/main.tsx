import React, { useState, useEffect, useRef, useMemo } from "react";
import ReactDOM from "react-dom";

const getJSON = async (url) => {
  let response = await fetch(url, {
    mode: "cors",
  });
  let text = await response.text();
  let json = JSON.parse(text);
  return json;
};

const postJSON = async (url, data?) => {
  let response = await fetch(url, {
    method: "POST",
    mode: "cors",
    headers: {
      "Content-Type": "application/json",
    },
    body: data ? JSON.stringify(data) : null,
  });
  let text = await response.text();
  let json = JSON.parse(text);
  return json;
};

const API_BASENAME = "http://localhost:3004";

const getJsonFromTodoService = async (url) => {
  let result = await getJSON(API_BASENAME + url);
  if (result.success) {
    return result.data;
  } else {
    throw new Error(result.message);
  }
};

const postJsonFromTodoService = async (url, data?) => {
  let result = await postJSON(API_BASENAME + url, data);
  if (result.success) {
    return result.data;
  } else {
    throw new Error(result.message);
  }
};

const getTodos = async (filterType = "all") => {
  return getJsonFromTodoService(`/todos?filter=${filterType}`);
};

const addTodo = async (content) => {
  let data = {
    content,
  };
  return postJsonFromTodoService(`/add_todo`, data);
};

const updateTodo = async (todoId, content) => {
  let data = {
    todo_id: todoId,
    content,
  };
  return postJsonFromTodoService(`/update_todo`, data);
};

const toggleTodo = async (todoId) => {
  let data = {
    todo_id: todoId,
  };
  return postJsonFromTodoService(`/toggle_todo`, data);
};

const removeTodo = async (todoId) => {
  let data = {
    todo_id: todoId,
  };
  return postJsonFromTodoService(`/remove_todo`, data);
};

const toggleAll = async () => {
  return postJsonFromTodoService(`/toggle_all`);
};

const clearCompleted = async () => {
  return postJsonFromTodoService(`/clear_completed`);
};

const FilterTypes = ["all", "active", "completed"];

const getFilterType = () => {
  return new URLSearchParams(location.hash.substring(1)).get("filter") || "all";
};

const App = () => {
  let [todos, setTodos] = useState([]);
  let [changeSignal, setSingal] = useState(0);
  let [text, setText] = useState("");
  let [filterType, setFilterType] = useState(getFilterType);

  let triggerGetTodos = () => {
    setSingal((n) => n + 1);
  };

  let handleToggle = async (todoId) => {
    try {
      await toggleTodo(todoId);
      triggerGetTodos();
    } catch (error) {
      alert(error.message);
    }
  };

  let handleRemove = async (todoId) => {
    try {
      await removeTodo(todoId);
      triggerGetTodos();
    } catch (error) {
      alert(error.message);
    }
  };

  let handleUpdate = async (todoId, content) => {
    try {
      await updateTodo(todoId, content);
      triggerGetTodos();
    } catch (error) {
      alert(error.message);
    }
  };

  let handleTextChange = (event) => {
    setText(event.target.value);
  };

  let handleAddTodo = async (event) => {
    if (event.keyCode !== 13) {
      return;
    }

    try {
      await addTodo(text);
      setText("");
      triggerGetTodos();
    } catch (error) {
      alert(error.message);
    }
  };

  let handleToggleAll = async () => {
    try {
      await toggleAll();
      setText("");
      triggerGetTodos();
    } catch (error) {
      alert(error.message);
    }
  };

  let handleClearCompleted = async () => {
    try {
      await clearCompleted();
      setText("");
      triggerGetTodos();
    } catch (error) {
      alert(error.message);
    }
  };

  useEffect(() => {
    let isOver = false;
    getTodos(filterType).then((todos) => {
      if (!isOver) {
        setTodos(todos);
      }
    });
    return () => {
      isOver = true;
    };
  }, [changeSignal, filterType]);

  useEffect(() => {
    let handleHashChange = () => {
      setFilterType(getFilterType());
    };
    window.addEventListener("hashchange", handleHashChange, false);
    return () => {
      window.removeEventListener("hashchange", handleHashChange, false);
    };
  });

  return (
    <>
      <header>
        <h1>Todo App</h1>
        <div>
          <input
            type='text'
            value={text}
            onChange={handleTextChange}
            onKeyUp={handleAddTodo}
          />
          <button onClick={handleToggleAll}>toggle all</button>
        </div>
      </header>
      <ul>
        {todos.map((todo) => {
          return (
            <Todo
              key={todo.id}
              todo={todo}
              onToggle={handleToggle}
              onUpdate={handleUpdate}
              onRemove={handleRemove}
            ></Todo>
          );
        })}
      </ul>
      {FilterTypes.map((filterType) => {
        return (
          <div key={filterType}>
            <a href={`#filter=${filterType}`}>{filterType}</a>
          </div>
        );
      })}
      <div>
        <button onClick={handleClearCompleted}>clear completed</button>
      </div>
    </>
  );
};

const Todo = ({ todo, onToggle, onRemove, onUpdate }) => {
  let [status, setStatus] = useState({
    isEditing: false,
  });

  let [text, setText] = useState("");

  let inputRef = useRef(null);

  let handleEnableEdit = () => {
    setStatus({
      ...status,
      isEditing: true,
    });
    setText(todo.content);
    if (inputRef.current) {
      setTimeout(() => {
        inputRef.current.focus();
      }, 0);
    }
  };

  let handleDisableEdit = () => {
    setStatus({
      ...status,
      isEditing: false,
    });
    setText("");
  };

  let handleSubmit = () => {
    let currentText = text;
    if (currentText === "") {
      handleRemove();
    } else {
      setText("");
      onUpdate(todo.id, currentText);
    }
    handleDisableEdit();
  };

  let handleToggle = () => {
    onToggle(todo.id);
  };

  let handleRemove = () => {
    onRemove(todo.id);
  };

  let handleTextChange = (event) => {
    setText(event.target.value);
  };

  let handleKeyUp = (event) => {
    if (event.keyCode === 13) {
      handleSubmit();
    } else if (event.keyCode === 27) {
      handleDisableEdit();
    }
  };

  let handleBlur = () => {
    if (text === todo.content) {
      handleDisableEdit();
    } else {
      handleSubmit();
    }
  };

  return (
    <li>
      {!status.isEditing && (
        <label
          onDoubleClick={handleEnableEdit}
          style={{ display: "inline-block", minWidth: 100 }}
        >
          {todo.content}
        </label>
      )}
      {status.isEditing && (
        <input
          style={{ display: "inline-block", minWidth: 100 }}
          type='text'
          value={text}
          onChange={handleTextChange}
          onBlur={handleBlur}
          onKeyUp={handleKeyUp}
          ref={inputRef}
        />
      )}
      <button onClick={handleToggle}>
        {!todo.completed ? "active" : "completed"}
      </button>
      <button onClick={handleRemove}>delete</button>
    </li>
  );
};

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById("root")
);
