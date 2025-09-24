#!/usr/bin/env python3
"""
GitHub Actions Status Checker using PyGithub
Much cleaner than using curl!
"""

import os
from github import Github, Auth
from datetime import datetime, timedelta

def check_github_actions():
    """Check the latest GitHub Actions runs for the repository."""
    
    # Get GitHub token from environment
    token = os.getenv('GITHUB_TOKEN')
    if not token:
        print("❌ GITHUB_TOKEN environment variable not set")
        return
    
    # Get repository from environment
    repo_name = os.getenv('GITHUB_REPOSITORY')
    if not repo_name:
        print("❌ GITHUB_REPOSITORY environment variable not set")
        return
    
    try:
        # Initialize GitHub client
        g = Github(auth=Auth.Token(token))
        repo = g.get_repo(repo_name)
        
        print(f"🔍 Checking GitHub Actions for {repo_name}")
        print("=" * 60)
        
        # Get latest workflow runs
        workflows = repo.get_workflows()
        print(f"📋 Found {workflows.totalCount} workflows")
        
        # Get recent runs
        runs = repo.get_workflow_runs()
        recent_runs = list(runs[:5])  # Get last 5 runs
        
        print(f"\n📊 Recent Workflow Runs:")
        print("-" * 60)
        
        for run in recent_runs:
            status_emoji = {
                'completed': '✅' if run.conclusion == 'success' else '❌',
                'in_progress': '🔄',
                'queued': '⏳',
                'cancelled': '⏹️'
            }.get(run.status, '❓')
            
            conclusion_text = f" ({run.conclusion})" if run.conclusion else ""
            created_at = run.created_at.strftime("%Y-%m-%d %H:%M:%S UTC")
            
            print(f"{status_emoji} {run.name}")
            print(f"   Status: {run.status}{conclusion_text}")
            print(f"   Created: {created_at}")
            print(f"   Commit: {run.head_sha[:8]}")
            print(f"   URL: {run.html_url}")
            print()
        
        # Check specific workflow status
        print("🔍 Detailed Status Check:")
        print("-" * 60)
        
        for run in recent_runs[:3]:  # Check last 3 runs
            print(f"\n📋 {run.name} (Run #{run.run_number})")
            
            # Get jobs for this run
            try:
                jobs = list(run.jobs())
                for job in jobs:
                    job_status = {
                        'completed': '✅' if job.conclusion == 'success' else '❌',
                        'in_progress': '🔄',
                        'queued': '⏳',
                        'cancelled': '⏹️'
                    }.get(job.status, '❓')
                    
                    conclusion_text = f" ({job.conclusion})" if job.conclusion else ""
                    print(f"  {job_status} {job.name}{conclusion_text}")
                    
                    # Get steps for failed jobs
                    if job.conclusion == 'failure':
                        steps = list(job.steps())
                        for step in steps:
                            if step.conclusion == 'failure':
                                print(f"    ❌ {step.name}")
            except Exception as e:
                print(f"  ❌ Error getting jobs: {e}")
        
        print("\n🎯 Summary:")
        print("-" * 60)
        
        # Count statuses
        status_counts = {}
        for run in recent_runs:
            status = run.status
            conclusion = run.conclusion
            key = f"{status}" + (f" ({conclusion})" if conclusion else "")
            status_counts[key] = status_counts.get(key, 0) + 1
        
        for status, count in status_counts.items():
            print(f"  {status}: {count} runs")
            
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    check_github_actions()
