import os
import re
from datetime import datetime

def is_memory_or_correction(text: str) -> bool:
    """
    Checks if the user input is asking to remember something or correcting a mistake.
    """
    text_lower = text.lower()
    
    # Keywords for remembering/memorizing
    memory_keywords = [
        r"remember",
        r"memorize",
        r"keep this in mind",
        r"note this",
        r"take note",
        r"always do",
        r"never do"
    ]
    
    # Keywords for corrections
    correction_keywords = [
        r"did .* wrong",
        r"that is wrong",
        r"that's wrong",
        r"don't do that",
        r"instead of",
        r"change that",
        r"change how",
        r"fix that",
        r"next time",
        r"made a mistake",
        r"incorrect",
        r"you should have",
        r"supposed to"
    ]
    
    # Combine and check
    for pattern in memory_keywords + correction_keywords:
        if re.search(pattern, text_lower):
            return True
            
    return False

def log_interaction(user_input: str, tool_calls: list = None, base_dir: str = ".agents/logs"):
    """
    Logs the user input if it contains memory or correction requests.
    """
    if not is_memory_or_correction(user_input):
        return False
        
    # Ensure directory exists
    os.makedirs(base_dir, exist_ok=True)
    
    # Generate filename based on current date
    today = datetime.now().strftime("%Y-%m-%d")
    filepath = os.path.join(base_dir, f"{today}.md")
    
    # Append to the file
    timestamp = datetime.now().strftime("%H:%M:%S")
    with open(filepath, "a", encoding="utf-8") as f:
        f.write(f"## Time: {timestamp}\n")
        f.write(f"**User Request:** {user_input}\n")
        if tool_calls:
            for tc in tool_calls:
                f.write(f"**System:** {tc}\n")
        f.write("\n")
        
    return True

if __name__ == "__main__":
    import sys
    import json
    
    # You can set this to whatever base directory you need
    LOG_DIR = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "logs"))
    
    if len(sys.argv) > 1 and sys.argv[1] == "--hook":
        # Hook mode: read JSON from stdin
        try:
            payload = json.load(sys.stdin)
            transcript_path = payload.get("transcriptPath")
            
            if transcript_path and os.path.exists(transcript_path):
                # Read the transcript file to find the latest USER_INPUT and collect tool calls
                latest_user_input = None
                latest_step_idx = -1
                session_tool_calls = []
                
                with open(transcript_path, "r", encoding="utf-8") as f:
                    for line in f:
                        if not line.strip(): continue
                        try:
                            step = json.loads(line)
                            step_type = step.get("type")
                            
                            if step_type == "USER_INPUT":
                                content = step.get("content", "")
                                # Extract content from <USER_REQUEST> if present
                                match = re.search(r"<USER_REQUEST>\n?(.*?)\n?</USER_REQUEST>", content, re.DOTALL)
                                if match:
                                    latest_user_input = match.group(1).strip()
                                else:
                                    latest_user_input = content.strip()
                                latest_step_idx = step.get("step_index", -1)
                            
                            elif step_type == "PLANNER_RESPONSE":
                                tcs = step.get("tool_calls", [])
                                for tc in tcs:
                                    name = tc.get("name", "unknown")
                                    args = tc.get("args", {})
                                    if not isinstance(args, dict):
                                        args = {}
                                    summary = args.get("toolSummary", args.get("toolAction", "do something"))
                                    if isinstance(summary, str):
                                        summary = summary.strip('"\'')
                                    session_tool_calls.append(f'Use tool "{name}" to {summary}')
                                    
                        except json.JSONDecodeError:
                            pass
                
                # Check if we already logged this step
                conversation_id = payload.get("conversationId", "default")
                tracker_file = os.path.join(LOG_DIR, f".last_logged_step_{conversation_id}")
                last_logged = -1
                if os.path.exists(tracker_file):
                    with open(tracker_file, "r") as f:
                        last_logged = int(f.read().strip() or "-1")
                        
                if latest_user_input and latest_step_idx > last_logged:
                    logged = log_interaction(latest_user_input, tool_calls=session_tool_calls, base_dir=LOG_DIR)
                    if logged:
                        # Update tracker
                        with open(tracker_file, "w") as f:
                            f.write(str(latest_step_idx))
            
            # Hook output must be a valid JSON object, e.g., {}
            print(json.dumps({}))
        except Exception as e:
            # Output empty JSON even on error to not break the agent
            print(json.dumps({}))
            
    elif len(sys.argv) > 1:
        text = " ".join(sys.argv[1:])
        logged = log_interaction(text, base_dir=LOG_DIR)
        if logged:
            print("Interaction logged to memory.")
        else:
            print("Interaction did not match memory or correction criteria.")
    else:
        print("Usage: python memory_logger.py \"your text here\"")
        print("       python memory_logger.py --hook (reads JSON from stdin)")
        print("Or enter interactive mode (type 'exit' to quit):")
        while True:
            try:
                user_input = input("> ")
                if user_input.strip().lower() in ['exit', 'quit']:
                    break
                
                if not user_input.strip():
                    continue
                    
                logged = log_interaction(user_input, base_dir=LOG_DIR)
                if logged:
                    print("-> Saved to memory log.")
            except EOFError:
                break
            except KeyboardInterrupt:
                break
